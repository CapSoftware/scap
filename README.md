<img src="./.github/banner.gif">

A Rust library to leverage native OS APIs for optimal performance and high-quality screen recordings. We use Apple's [ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit) on macOS and [Graphics.Capture](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture?view=winrt-22621) APIs on Windows. Linux support is planned but not underway yet, PRs welcome!

**🚧 WIP. Unsuitable for production use, APIs are being iterated on.**

[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/SC468DK4du)

---

## features

1. Cross-platform support: Windows and Mac now, Linux soon.
2. Check for support and user permissions.
3. Utilize native OS APIs for screen capture.
4. Different capture modes: audio, display or window.

## contributing

I found most of Rust's tooling around screen capture either non-performant, outdated or very platform-specific. This project is my attempt to change that. It's early days and the code is fairly simple, I'll gladly accept any contributions/PRs.

If you'd like to chip in, here's a kickstart guide:

1. Clone the repo and run it with `cargo run`.
2. Explore the API and library code in `lib.rs`.
3. Platform-specific code is in the `win` and `mac` modules.
4. There's a small program in `main.rs` that "consumes" the library for dev-testing.

## usage

```rust
use scap::{Options, Recorder};

fn main() {
    // Check if the platform is supported
    let supported = scap::is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return;
    } else {
        println!("✅ Platform supported");
    }

    // Check if we have permission to capture the screen
    let has_permission = scap::has_permission();
    if !has_permission {
        println!("❌ Permission not granted");
        return;
    } else {
        println!("✅ Permission granted");
    }

    // Get recording targets (WIP)
    let targets = scap::get_targets();
    println!("🎯 Targets: {:?}", targets);

    // Create Options
    let options = Options {
        fps: 60,
        targets,
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
    };

    // Create Recorder
    let mut recorder = Recorder::init(options);

    // Start Capture
    recorder.start_capture();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Stop Capture
    recorder.stop_capture();
}
```

## roadmap

-   [x] Check for support and user permissions.
-   [x] Capture frames
-   [x] Capture targets: monitors, windows, region and audio.
-   [ ] Encoding: encode frames to file.

## license

The code in this repository is open-sourced under the MIT license. However, it may rely on dependencies that are licensed differently. Please consult their documentations for exact terms.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Pranav2612000"><img src="https://avatars.githubusercontent.com/u/20909078?v=4?s=100" width="100px;" alt="Pranav Joglekar"/><br /><sub><b>Pranav Joglekar</b></sub></a><br /><a href="#code-Pranav2612000" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://dev-rohan.in"><img src="https://avatars.githubusercontent.com/u/48467821?v=4?s=100" width="100px;" alt="Rohan Punjani"/><br /><sub><b>Rohan Punjani</b></sub></a><br /><a href="#code-RohanPunjani" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## credits

This project builds on top of the fabulous work done by [@svtlabs](https://github.com/svtlabs) and [@NiiightmareXD](https://github.com/NiiightmareXD).

-   [screencapturekit-rs](https://github.com/svtlabs/screencapturekit-rs): Rust bindings for Apple's ScreencaptureKit API.
-   [windows-capture](https://github.com/NiiightmareXD/windows-capture): Rust library for Windows.Graphics.Capture.
