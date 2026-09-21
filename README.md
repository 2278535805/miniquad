# Miniquad

[![Github Actions](https://github.com/not-fl3/miniquad/workflows/Cross-compile/badge.svg)](https://github.com/not-fl3/miniquad/actions?query=workflow%3A)
[![Docs](https://docs.rs/miniquad/badge.svg?version=0.3.13)](https://docs.rs/miniquad/0.3.13/miniquad/index.html)
[![Crates.io version](https://img.shields.io/crates/v/miniquad.svg)](https://crates.io/crates/miniquad)
[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/WfEp6ut)
[![Matrix](https://img.shields.io/matrix/quad-general:matrix.org?label=matrix%20chat)](https://matrix.to/#/#quad-general:matrix.org)

Miniquad is a manifestation of a dream in a world where we do not need a deep dependencies tree and thousands lines of code to draw things with a computer.

Miniquad aims to provide a graphics abstraction that works the same way on any platform with a GPU, being as light weight as possible while covering as many machines as possible. 

## Supported Platforms

* Windows, OpenGL 3, OpenGL 2.2;
* Linux, OpenGL 2.2, OpenGL 3, GLES 2, GLES 3;
* macOS, OpenGL 3, Metal;
* iOS, GLES 2, GLES 3, Metal;
* WASM, WebGL 1 - tested on iOS Safari, Firefox, Chrome;
* Android, GLES 2, GLES 3.

## Examples

![Imgur](https://i.imgur.com/TRI50rk.gif)

[examples/quad.rs](https://github.com/not-fl3/miniquad/blob/master/examples/quad.rs): [web demo](https://not-fl3.github.io/miniquad-samples/quad.html)<br/>
[examples/offscreen.rs](https://github.com/not-fl3/miniquad/blob/master/examples/offscreen.rs): [web demo](https://not-fl3.github.io/miniquad-samples/offscreen.html)<br/>

[PonasKovas/miniquad-mandelbrot](https://github.com/PonasKovas/miniquad-mandelbrot): [web demo](https://ponaskovas.github.io/miniquad-mandelbrot-wasm-demo/)

# Building examples

## Linux

```bash
cargo run --example quad
```

On NixOS Linux you can use [`shell.nix`](shell.nix) to start a development
environment where Miniquad can be built and run.

## Windows

```bash
# both MSVC and GNU target is supported:
rustup target add x86_64-pc-windows-msvc
# or
rustup target add x86_64-pc-windows-gnu

cargo run --example quad
```

### Windows headless rendering

Set `Conf::headless` to `true` to render into an EGL pbuffer using
[ANGLE](https://github.com/google/angle)'s OpenGL ES 3 / D3D11 backend.
This path does not create an application window, HWND/HDC or WGL context.
Deploy `libEGL.dll` and `libGLESv2.dll` from the same D3D11-enabled ANGLE build
beside the executable, along with any dependencies of that build (such as
`d3dcompiler_47.dll`). DLL architecture must match the executable.

```rust
let conf = miniquad::conf::Conf {
    headless: true,
    platform: miniquad::conf::Platform {
        // Use CPU rendering for remote desktop / machines without a usable GPU.
        windows_egl: miniquad::conf::WindowsEgl::D3D11Warp,
        ..Default::default()
    },
    ..Default::default()
};
```

The default `D3D11WithWarpFallback` tries hardware first and falls back to WARP
during initialization. `D3D11` requires hardware; `D3D11Warp` always uses the CPU.
The selected renderer is printed at startup. WARP trades GPU performance for
independence from hardware graphics drivers. This does not implement recovery
from device loss after initialization.

ANGLE is asked for the newest OpenGL ES context it can provide (3.0, 3.1 or
3.2). Shader sources are adapted to the context automatically: the `#version`
directive is optional. Shaders written in the old `attribute`/`varying` style
are compiled as GLSL ES 1.00, modern `in`/`out` shaders as the newest GLSL ES
version the context supports, and desktop directives (for example
`#version 330` or `#version 450`) are translated to GLSL ES. Fragment shaders
get a default float precision when they do not declare one. Shader features
still need to be supported by the GLES version in use. Raw GL
calls must be supported by GLES 3 / the available extensions. `set_window_size`
recreates the pbuffer while keeping the context and GL resources. Window/input
requests have no effect, the clipboard is empty, and DPI scale is 1. A pbuffer
has no vsync, so `swap_interval` is ignored.
`blocking_event_loop` is supported; with it, `sleep_interval_ms` requests periodic
redraws. Read results through `texture_read_pixels` or `glReadPixels` before exiting.

### Linux headless rendering

Set `Conf::headless` to `true` to render into an EGL pbuffer with a desktop
OpenGL context, without X11 or Wayland. This requires `libEGL` and a driver
supporting either `EGL_EXT_platform_device` or
`EGL_MESA_platform_surfaceless`, and a desktop OpenGL 3.x or newer context.
The environment variable `MINIQUAD_EGL_DEVICE` selects the device index used
with `eglQueryDevicesEXT` (defaults to 0). `set_window_size` recreates the
pbuffer while keeping the context and GL resources, and `sample_count` is
honored when a matching multisampled pbuffer config is available.
Window/input requests have no effect, the clipboard is empty, and DPI scale is 1.
`blocking_event_loop` is supported; with it, `sleep_interval_ms` requests periodic
redraws. Read results through `texture_read_pixels` or `glReadPixels` before exiting.

## WASM

```bash
rustup target add wasm32-unknown-unknown
cargo build --example quad --target wasm32-unknown-unknown
```

And then use the following .html to load .wasm:

<details><summary>index.html</summary>

```html
<html lang="en">

<head>
    <meta charset="utf-8">
    <title>TITLE</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            background: black;
            z-index: 0;
        }
    </style>
</head>

<body>
    <canvas id="glcanvas" tabindex='1'></canvas>
    <!-- Minified and statically hosted version of https://github.com/not-fl3/miniquad/blob/master/native/sapp-wasm/js/gl.js -->
    <script src="https://not-fl3.github.io/miniquad-samples/gl.js"></script>
    <script>load("quad.wasm");</script> <!-- Your compiled wasm file -->
</body>

</html>
```
</details>

One of the ways to server static .wasm and .html:

```no-run
cargo install basic-http-server
basic-http-server .
```

## Android

Recommended way to build for android is using Docker.<br/>
miniquad uses slightly modifed version of `cargo-apk`

```no-run
docker run --rm -v $(pwd)":/root/src" -w /root/src notfl3/cargo-apk cargo quad-apk build --example quad
```

APK file will be in `target/android-artifacts/(debug|release)/apk`

With "log-impl" enabled all log calls will be forwarded to adb console.
No code modifications for Android required, everything should just works.

## iOS

To run on the simulator:

```no-run
mkdir MyGame.app
cargo build --target x86_64-apple-ios --release
cp target/release/mygame MyGame.app
# only if the game have any assets
cp -r assets MyGame.app
cat > MyGame.app/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
<key>CFBundleExecutable</key>
<string>mygame</string>
<key>CFBundleIdentifier</key>
<string>com.mygame</string>
<key>CFBundleName</key>
<string>mygame</string>
<key>CFBundleVersion</key>
<string>1</string>
<key>CFBundleShortVersionString</key>
<string>1.0</string>
</dict>
</plist>
EOF

xcrun simctl install booted MyGame.app/
xcrun simctl launch booted com.mygame
```

For details and instructions on provisioning for real iphone, check [https://macroquad.rs/articles/ios/](https://macroquad.rs/articles/ios/)

## Cross Compilation

```bash

# windows target from linux host:
# this is how windows builds are tested from linux machine:
rustup target add x86_64-pc-windows-gnu
cargo run --example quad --target x86_64-pc-windows-gnu
```

# Goals

* Fast compilation time. Right now it is ~5s from "cargo clean" for both desktop and web.

* Cross platform. Amount of platform specific user code required should be kept as little as possible.

* Low-end devices support.

* Hackability. Working on your own game, highly probable some hardware incompability will be found. Working around that kind of bugs should be easy, implementation details should not be hidden under layers of abstraction.

* Forkability. Each platform implementation is, usually, just one pure Rust file. And this file is very copy-paste friendly - it doesnt use any miniquad specific abstractions. It is very easy to just copy some part of miniquad's platform implementation and use it standalone.

# Non-goals

* Ultimate type safety. Library should be entirely safe in Rust's definition of safe - no UB or memory unsafety. But correct GPU state is not type guaranteed. Feel free to provide safety abstraction in the user code then!

* High-end API, like Vulkan/DirectX 12. Take a look on [gfx-rs](https://github.com/gfx-rs/gfx) or [vulkano](https://github.com/vulkano-rs/vulkano) instead!

# Platinum sponsors

Miniquad is supported by:

[SourceGear](https://www.sourcegear.com/)
