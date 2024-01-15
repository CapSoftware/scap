use std::sync::atomic::AtomicU8;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use pipewire as pw;
use pw::properties;
use pw::spa;
use pw::spa::format::FormatProperties;
use pw::spa::format::MediaSubtype;
use pw::spa::format::MediaType;
use pw::spa::Direction;
use pw::spa::param::video::VideoFormat;

use crate::{capturer::Options, frame::Frame};

use self::error::LinCapError;

mod error;

static CAPTURER_STATE: AtomicU8 = AtomicU8::new(0);

#[derive(Clone)]
struct ListenerUserData {
    pub tx: mpsc::Sender<Frame>,
    pub format: spa::param::video::VideoInfoRaw,
}

fn pipewire_capturer(options: Options, tx: mpsc::Sender<Frame>) -> Result<(), LinCapError> {
    assert!(!options.targets.is_empty());

    pw::init();

    let mainloop = pw::MainLoop::new()?;
    let context = pw::Context::new(&mainloop)?;
    let core = context.connect(None)?;

    let user_data = ListenerUserData {
        tx,
        format: Default::default(),
    };

    let stream = pw::stream::Stream::new(
        &core,
        "scap",
        properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Screen",
        },
    )?;

    let _listener = stream
        .add_local_listener_with_user_data(user_data.clone())
        .state_changed(|old, new| {
            println!(
                "linux::pipewire::stream: State changed: {:?} -> {:?}",
                old, new
            );
        })
        .param_changed(|_, id, user_data: &mut ListenerUserData, param| {
            let Some(param) = param else {
                return;
            };
            if id != pw::spa::param::ParamType::Format.as_raw() {
                return;
            }
            let (media_type, media_subtype) =
                match pw::spa::param::format_utils::parse_format(param) {
                    Ok(v) => v,
                    Err(_) => return,
                };

            if media_type != MediaType::Video || media_subtype != MediaSubtype::Raw {
                return;
            }

            user_data
                .format
                .parse(param)
                .expect("Failed to parse parameter");

            println!("Got video format:");
            println!(
                "  format: {} ({:?})",
                user_data.format.format().as_raw(),
                user_data.format.format()
            );
            println!(
                "  size: {}x{}",
                user_data.format.size().width,
                user_data.format.size().height
            );
            println!(
                "  framerate: {}/{}",
                user_data.format.framerate().num,
                user_data.format.framerate().denom
            );
        })
        .process(|stream, user_data| match stream.dequeue_buffer() {
            None => println!("Out of buffers"),
            Some(mut buffer) => {
                let datas = buffer.datas_mut();
                if datas.is_empty() {
                    return;
                }

                if let Some(frame_data) = (&mut datas[0]).data() {
                    match user_data.format.format() {
                        VideoFormat::RGBx => {
                            if let Err(e) = user_data.tx.send(Frame::RGBx(frame_data.to_vec())) {
                                println!("{e}");
                            }
                        }
                        _ => panic!("Unsupported frame format received"),
                    }
                }

            }
        })
        .register()?;

    let obj = pw::spa::pod::object!(
        pw::spa::utils::SpaTypes::ObjectParamFormat,
        pw::spa::param::ParamType::EnumFormat,
        pw::spa::pod::property!(FormatProperties::MediaType, Id, MediaType::Video),
        pw::spa::pod::property!(FormatProperties::MediaSubtype, Id, MediaSubtype::Raw),
        pw::spa::pod::property!(
            FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::RGBA,
            pw::spa::param::video::VideoFormat::RGBx,
            pw::spa::param::video::VideoFormat::BGRx,
            pw::spa::param::video::VideoFormat::YUY2,
            pw::spa::param::video::VideoFormat::I420,
        ),
        pw::spa::pod::property!(
            FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            pw::spa::utils::Rectangle {
                // Default
                width: 128,
                height: 128,
            },
            pw::spa::utils::Rectangle {
                // Min
                width: 1,
                height: 1,
            },
            pw::spa::utils::Rectangle {
                // Max
                width: 4096,
                height: 4096,
            }
        ),
        pw::spa::pod::property!(
            FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            pw::spa::utils::Fraction { num: 25, denom: 1 },
            pw::spa::utils::Fraction { num: 0, denom: 1 },
            pw::spa::utils::Fraction {
                num: 1000,
                denom: 1
            }
        ),
    );

    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .unwrap()
    .0
    .into_inner();

    let mut params = [pw::spa::pod::Pod::from_bytes(&values).unwrap()];

    stream
        .connect(
            Direction::Input,
            Some(options.targets[0].id),
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )?;

    while CAPTURER_STATE.load(std::sync::atomic::Ordering::Relaxed) == 0 {
        std::thread::sleep(Duration::from_millis(10));
    }

    // User has called Capturer::start() and we start the main loop
    while CAPTURER_STATE.load(std::sync::atomic::Ordering::Relaxed) == 1 {
        mainloop.iterate(Duration::from_millis(100));
    }

    Ok(())
}

pub struct LinuxCapturer {
    capturer_join_handle: Option<JoinHandle<()>>,
}

impl LinuxCapturer {
    // TODO: Error handling
    pub fn new(options: &Options, tx: mpsc::Sender<Frame>) -> Self {
        // TODO: Fix this hack
        let options = Options {
            fps: options.fps,
            show_cursor: options.show_cursor,
            show_highlight: options.show_highlight,
            output_type: options.output_type,
            targets: options.targets.clone(),
            excluded_targets: None,
        };
        let capturer_join_handle = std::thread::spawn(move || {
            pipewire_capturer(options, tx);
        });

        Self {
            capturer_join_handle: Some(capturer_join_handle),
        }
    }

    pub fn start_capture(&self) {
        CAPTURER_STATE.store(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn stop_capture(&mut self) {
        CAPTURER_STATE.store(2, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.capturer_join_handle.take() {
            handle.join().expect("Failed to join capturer thread");
        }
    }
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> LinuxCapturer {
    LinuxCapturer::new(options, tx)
}
