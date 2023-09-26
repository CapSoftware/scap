use screencapturekit::{sc_display, sc_shareable_content, sc_stream, sc_stream_configuration};

fn main() -> Result<(), ()> {
    let content = sc_shareable_content::SCShareableContent::current();

    content.applications.iter().for_each(|app| {
        println!("App: {:?}", app);
    });

    content.displays.iter().for_each(|display| {
        println!("Display: {:?}", display);
    });

    content.windows.iter().for_each(|window| {
        println!("Window: {:?}", window);
    });

    let mut stream_config = sc_stream_configuration::SCStreamConfiguration::default();

    stream_config.shows_cursor = false;
    stream_config.width = 1920;
    stream_config.height = 1080;
    stream_config.captures_audio = false;
    stream_config.scales_to_fit = true;

    let stream = sc_stream::SCStream::new(filter, stream_config, handler);

    // print!("Display: {:?}", display);

    Ok(())
}
