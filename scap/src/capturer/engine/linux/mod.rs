use std::{
    mem::size_of,
    sync::{
        atomic::AtomicU8,
        mpsc::{self, sync_channel, SyncSender},
    },
    thread::JoinHandle,
    time::Duration,
};

use ashpd::{
    desktop::screencast::{CursorMode, Screencast},
    WindowIdentifier,
};
use pipewire as pw;
use pw::{
    properties,
    spa::{
        self,
        format::{FormatProperties, MediaSubtype, MediaType},
        param::{video::VideoFormat, ParamType},
        pod::{Pod, Property},
        sys::{
            spa_buffer, spa_meta_header, SPA_META_Header, SPA_PARAM_META_size, SPA_PARAM_META_type,
        },
        utils::SpaTypes,
        Direction,
    },
    stream::{StreamRef, StreamState},
};

use crate::{
    capturer::Options,
    frame::{BGRxFrame, Frame, RGBFrame, RGBxFrame, XBGRFrame},
};

use self::error::LinCapError;

mod error;

static CAPTURER_STATE: AtomicU8 = AtomicU8::new(0);

#[derive(Clone)]
struct ListenerUserData {
    pub tx: mpsc::Sender<Frame>,
    pub format: spa::param::video::VideoInfoRaw,
}

fn param_changed_callback(
    _stream: &StreamRef,
    id: u32,
    user_data: &mut ListenerUserData,
    param: Option<&Pod>,
) {
    let Some(param) = param else {
        return;
    };
    if id != pw::spa::param::ParamType::Format.as_raw() {
        return;
    }
    let (media_type, media_subtype) = match pw::spa::param::format_utils::parse_format(param) {
        Ok(v) => v,
        Err(_) => return,
    };

    if media_type != MediaType::Video || media_subtype != MediaSubtype::Raw {
        return;
    }

    user_data
        .format
        .parse(param)
        // TODO: Tell library user of the error
        .expect("Failed to parse format parameter");

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
}

fn state_changed_callback(old: StreamState, new: StreamState) {
    println!(
        "linux::pipewire::stream: State changed: {:?} -> {:?}",
        old, new
    );
}

unsafe fn get_timestamp(buffer: *mut spa_buffer) -> i64 {
    let n_metas = (*buffer).n_metas;
    if n_metas > 0 {
        let mut meta_ptr = (*buffer).metas;
        let metas_end = (*buffer).metas.wrapping_add(n_metas as usize);
        while meta_ptr != metas_end {
            if (*meta_ptr).type_ == SPA_META_Header {
                let meta_header: &mut spa_meta_header =
                    &mut *((*meta_ptr).data as *mut spa_meta_header);
                return meta_header.pts;
            }
            meta_ptr = meta_ptr.wrapping_add(1);
        }
        0
    } else {
        0
    }
}

fn process_callback(stream: &StreamRef, user_data: &mut ListenerUserData) {
    let buffer = unsafe { stream.dequeue_raw_buffer() };
    if !buffer.is_null() {
        'outside: {
            let buffer = unsafe { (*buffer).buffer };
            if buffer.is_null() {
                break 'outside;
            }
            let timestamp = unsafe { get_timestamp(buffer) };

            let n_datas = unsafe { (*buffer).n_datas };
            if n_datas < 1 {
                return;
            }
            let frame_size = user_data.format.size();
            let frame_data: Vec<u8> = unsafe {
                std::slice::from_raw_parts(
                    (*(*buffer).datas).data as *mut u8,
                    (*(*buffer).datas).maxsize as usize,
                )
                .to_vec()
            };

            if let Err(e) = match user_data.format.format() {
                VideoFormat::RGBx => user_data.tx.send(Frame::RGBx(RGBxFrame {
                    display_time: timestamp as u64,
                    width: frame_size.width as i32,
                    height: frame_size.height as i32,
                    data: frame_data,
                })),
                VideoFormat::RGB => user_data.tx.send(Frame::RGB(RGBFrame {
                    display_time: timestamp as u64,
                    width: frame_size.width as i32,
                    height: frame_size.height as i32,
                    data: frame_data,
                })),
                VideoFormat::xBGR => user_data.tx.send(Frame::XBGR(XBGRFrame {
                    display_time: timestamp as u64,
                    width: frame_size.width as i32,
                    height: frame_size.height as i32,
                    data: frame_data,
                })),
                VideoFormat::BGRx => user_data.tx.send(Frame::BGRx(BGRxFrame {
                    display_time: timestamp as u64,
                    width: frame_size.width as i32,
                    height: frame_size.height as i32,
                    data: frame_data,
                })),
                _ => panic!("Unsupported frame format received"),
            } {
                eprintln!("{e}");
            }
        }
    } else {
        eprintln!("Out of buffers");
    }

    unsafe { stream.queue_raw_buffer(buffer) };
}

// TODO: Format negotiation
fn pipewire_capturer(
    options: Options,
    tx: mpsc::Sender<Frame>,
    ready_sender: &SyncSender<bool>,
    stream_id: u32,
) -> Result<(), LinCapError> {
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
        .state_changed(state_changed_callback)
        .param_changed(param_changed_callback)
        .process(process_callback)
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
            pw::spa::param::video::VideoFormat::RGBA,
            pw::spa::param::video::VideoFormat::RGBx,
            pw::spa::param::video::VideoFormat::BGRx,
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
            pw::spa::utils::Fraction {
                num: options.fps,
                denom: 1
            },
            pw::spa::utils::Fraction { num: 0, denom: 1 },
            pw::spa::utils::Fraction {
                num: 1000,
                denom: 1
            }
        ),
    );

    let metas_obj = pw::spa::pod::object!(
        SpaTypes::ObjectParamMeta,
        ParamType::Meta,
        Property::new(
            SPA_PARAM_META_type,
            pw::spa::pod::Value::Id(pw::spa::utils::Id(SPA_META_Header))
        ),
        Property::new(
            SPA_PARAM_META_size,
            pw::spa::pod::Value::Int(size_of::<pw::spa::sys::spa_meta_header>() as i32)
        ),
    );

    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )?
    .0
    .into_inner();
    let metas_values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(metas_obj),
    )?
    .0
    .into_inner();

    let mut params = [
        pw::spa::pod::Pod::from_bytes(&values).unwrap(),
        pw::spa::pod::Pod::from_bytes(&metas_values).unwrap(),
    ];

    stream.connect(
        Direction::Input,
        Some(stream_id),
        pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
        &mut params,
    )?;

    ready_sender.send(true)?;

    while CAPTURER_STATE.load(std::sync::atomic::Ordering::Relaxed) == 0 {
        std::thread::sleep(Duration::from_millis(10));
    }

    // User has called Capturer::start() and we start the main loop
    while CAPTURER_STATE.load(std::sync::atomic::Ordering::Relaxed) == 1 {
        mainloop.iterate(Duration::from_millis(100));
    }

    Ok(())
}

async fn get_screencast_stream(_options: &Options) -> Result<Option<u32>, LinCapError> {
    let proxy = Screencast::new().await?;
    let session = proxy.create_session().await?;

    proxy
        .select_sources(
            &session,
            if _options.show_cursor {
                ashpd::desktop::screencast::CursorMode::Embedded
            } else {
                CursorMode::Hidden
            },
            proxy.available_source_types().await?,
            false,
            None,
            ashpd::desktop::screencast::PersistMode::DoNot,
        )
        .await?;

    let response = proxy
        .start(&session, &WindowIdentifier::default())
        .await?
        .response()?;

    for stream in response.streams() {
        // Just return the first stream we get
        return Ok(Some(stream.pipe_wire_node_id()));
    }

    Ok(None)
}

pub struct LinuxCapturer {
    capturer_join_handle: Option<JoinHandle<Result<(), LinCapError>>>,
}

impl LinuxCapturer {
    // TODO: Error handling
    pub fn new(options: &Options, tx: mpsc::Sender<Frame>) -> Self {
        let Some(stream_id) = async_std::task::block_on(get_screencast_stream(options))
            .expect("Failed to get stream id")
        else {
            panic!("No stream available");
        };

        // TODO: Fix this hack
        let options = Options {
            fps: options.fps,
            show_cursor: options.show_cursor,
            show_highlight: options.show_highlight,
            output_type: options.output_type,
            targets: options.targets.clone(),
            excluded_targets: None,
            source_rect: None,
        };
        let (ready_sender, ready_recv) = sync_channel(1);
        let capturer_join_handle = std::thread::spawn(move || {
            let res = pipewire_capturer(options, tx, &ready_sender, stream_id);
            if res.is_err() {
                ready_sender.send(false)?;
            }
            res
        });

        if !ready_recv.recv().expect("Failed to receive") {
            panic!("Failed to setup capturer");
        }

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
            if let Err(e) = handle.join().expect("Failed to join capturer thread") {
                eprintln!("Error occured capturing: {e}");
            }
        }
    }
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> LinuxCapturer {
    LinuxCapturer::new(options, tx)
}
