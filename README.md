# Cypher

A modern, cross-platform, and high-performance library designed for screen capturing tasks. Cypher leverages the native operating system APIs; on macOS, it utilizes the [ScreencaptureKit](https://developer.apple.com/documentation/screencapturekit), and on Windows, [Windows.Graphics.Capture](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture?view=winrt-22621) to ensure optimal performance and quality.

> 🚧 Work-in-progress. Unsuitable for consumption at the moment.

## Contributing

I found the Rust ecosystems tooling around screen capture either non-performant (macOS) or outdated (Linux/Windows). This is my attempt to fix that. It's an early project and the code is fairly simple. I'll happily accept any help/contributions. If you'd like to, here's a few pointers:

1. Clone the repo and run it with `cargo run`.
2. Library code and API is in `lib.rs`.
3. All the platform-specific code is in the `win` and `mac` modules.
4. There's a small program in `main.rs` that "consumes" the library for testing in development.

## Features

1. Cross-platform support (macOS and Windows: wip, linux: soon after)
2. Check for support and user permissions.
3. Utilize native OS APIs for screen capture.
4. Different capture modes: display, windows or region.

## License

The code in this repository is open-sourced under the MIT license. However it might rely on dependencies that are licensed differently. Please consult their documentations for exact terms.
