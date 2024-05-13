![Github banner](./.github/banner.gif)

A Rust library for high-quality screen recordings that leverages native OS APIs for optimal performance: Apple's [ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit) on macOS, [Graphics.Capture](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture?view=winrt-22621) APIs on Windows and [Pipewire](https://pipewire.org) on Linux.

**🚧 WIP. Unsuitable for production use, APIs are being iterated on.**

[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/SC468DK4du)

---

## features

1. Cross-platform support: Windows, Mac and Linux!
2. Check for support and user permissions.
3. Utilize native OS APIs for screen capture.
4. Different capture modes: Display or Windows.

## contributing

We found most of Rust's tooling around screen capture either non-performant, outdated or very platform-specific. This project is our attempt to change that. Contributions, PRs and Issues are most welcome!

If you'd like to develop, here's a kickstart guide:

1. Clone the repo and run it with `cargo run`.
2. Explore the API and library code in [lib.rs](./scap/src/lib.rs).
3. Platform-specific code is in the `win`, `mac` and `linux` modules.
4. There's a small program in [main.rs](./scap/src/main.rs) that "consumes" the library for dev-testing.

## usage

```rust
use scap::{
    capturer::{CGPoint, CGRect, CGSize, Capturer, Options},
    frame::Frame,
};

fn main() {
    // Check if the platform is supported
    let supported = scap::is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return;
    } else {
        println!("✅ Platform supported");
    }

    // Check if we have permission to capture screen
    // If we don't, request it.
    if !scap::has_permission() {
        println!("❌ Permission not granted. Requesting permission...");
        if !scap::request_permission() {
            println!("❌ Permission denied");
            return;
        }
    }
    println!("✅ Permission granted");

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
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        source_rect: Some(CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: 2000.0,
                height: 1000.0,
            },
        }),
        ..Default::default()
    };

    // Create Recorder
    let mut capturer = Capturer::new(options);

    // Start Capture
    capturer.start_capture();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Stop Capture
    capturer.stop_capture();
}
```

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
      <td align="center" valign="top" width="14.28%"><a href="https://www.sid.me"><img src="https://avatars.githubusercontent.com/u/30227512?v=4?s=100" width="100px;" alt="Siddharth"/><br /><sub><b>Siddharth</b></sub></a><br /><a href="#code-clearlysid" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/NiiightmareXD"><img src="https://avatars.githubusercontent.com/u/90005793?v=4?s=100" width="100px;" alt="NiiightmareXD"/><br /><sub><b>NiiightmareXD</b></sub></a><br /><a href="#code-NiiightmareXD" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://bringeber.dev"><img src="https://avatars.githubusercontent.com/u/83474682?v=4?s=100" width="100px;" alt="MAlba124"/><br /><sub><b>MAlba124</b></sub></a><br /><a href="#code-MAlba124" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://peerlist.io/anubhavitis"><img src="https://avatars.githubusercontent.com/u/26124625?v=4?s=100" width="100px;" alt="Anubhav Singhal"/><br /><sub><b>Anubhav Singhal</b></sub></a><br /><a href="#code-anubhavitis" title="Code">💻</a></td>
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
