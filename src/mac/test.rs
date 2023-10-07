    // check for screen capture permission
    let access = ScreenCaptureAccess::default();
    let access = access.preflight();

    // if access isnt true, log it and return
    if !access {
        println!("screencapture access not granted");
        return;
    }

		println!("Pixel format type: {}", pixel_format_type); // 875704438 - yuv420v