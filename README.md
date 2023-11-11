# scap

modern, cross-platform, high-performance library built for screen capturing duties.

> 🚧 Work-in-progress. Unsuitable for production use at the moment.

scap leverages native operating system APIs for optimal performance and quality — Apple's [ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit) on macOS, [Windows.Graphics.Capture](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture?view=winrt-22621) APIs on Windows. Linux support is planned, but not currently being worked on. PRs welcome!

## features.

1. Cross-platform support: Windows and Mac now, Linux soon.
2. Check for support and user permissions.
3. Utilize native OS APIs for screen capture.
4. Different capture modes: audio, display or window.

## contributing.

I found most of Rust's tooling around screen capture either non-performant, outdated or very platform-specific. This project is my attempt to change that. It's early days and the code is fairly simple, I'll gladly accept any contributions/PRs.

If you'd like to chip in, here's a kickstart guide:

1. Clone the repo and run it with `cargo run`.
2. Explore the API and library code in `lib.rs`.
3. Platform-specific code is in the `win` and `mac` modules.
4. There's a small program in `main.rs` that "consumes" the library for dev-testing.

## roadmap.

-   [x] Check for support and user permissions.
-   [x] Capture frames
-   [x] Capture targets: monitors, windows, region and audio.
-   [ ] Encoding: encode frames to file.

## license.

The code in this repository is open-sourced under the MIT license. However, it may rely on dependencies that are licensed differently. Please consult their documentations for exact terms.

## credits.

This project builds on top of the fabulous work done by [@svtlabs](https://github.com/svtlabs) and [@NiiightmareXD](https://github.com/NiiightmareXD).

-   [screencapturekit-rs](https://github.com/svtlabs/screencapturekit-rs): Rust bindings for Apple's ScreencaptureKit API.
-   [windows-capture](https://github.com/NiiightmareXD/windows-capture): Rust library for Windows.Graphics.Capture.
