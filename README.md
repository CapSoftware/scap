<img src=".github/banner.png" alt="Scap - Rust Screen Capture Library" />

A modern, cross-platform, and high-performance library designed for screen capturing tasks. Scap leverages the native operating system APIs to ensure optimal performance and quality. On macOS it uses Apple's [ScreencaptureKit](https://developer.apple.com/documentation/screencapturekit) and on Windows it uses [Windows.Graphics.Capture](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture?view=winrt-22621) namespace.

> 🚧 Work-in-progress. Unsuitable for production use at the moment.

## Features

1. Cross-platform support: Windows and Mac now, Linux soon.
2. Check for support and user permissions.
3. Utilize native OS APIs for screen capture.
4. Different capture modes: audio, display or window.

## Contributing

I found the Rust's tooling around screen capture either non-performant, outdated or platform-specific. This project is my attempt to change that. It is early days and the code is fairly simple. I'll gladly accept any help or contributions.

Here's a few pointers if you'd like to chime in:

1. Clone the repo and run it with `cargo run`.
2. Explore the library code and API in `lib.rs`.
3. All the platform-specific code is in the `win` and `mac` modules.
4. There's a small program in `main.rs` that "consumes" the library for testing in development.

## Roadmap

-   [x] Check for support and user permissions.
-   [ ] Capture frames
-   [ ] Capture targets: monitors, windows, region and audio.
-   [ ] Encoding: encode frames to file.

## License

The code in this repository is open-sourced under the MIT license. However it might rely on dependencies that are licensed differently. Please consult their documentations for exact terms.

## Credits

This project builds on top of the fabulous work done by [@svtlabs](https://github.com/svtlabs) and [@NiiightmareXD](https://github.com/NiiightmareXD).

-   [screencapturekit-rs](https://github.com/svtlabs/screencapturekit-rs): Rust bindings for Apple's ScreencaptureKit API.
-   [windows-capture](https://github.com/NiiightmareXD/windows-capture): Rust library for Windows.Graphics.Capture.
