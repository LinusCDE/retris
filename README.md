# retris

[![rm1](https://img.shields.io/badge/rM1-supported-green)](https://remarkable.com/store/remarkable)
[![rm2](https://img.shields.io/badge/rM2-supported-green)](https://remarkable.com/store/remarkable-2)
[![opkg](https://img.shields.io/badge/OPKG-retris-blue)](https://github.com/toltec-dev/toltec)
[![launchers](https://img.shields.io/badge/Launchers-supported-green)](https://github.com/reHackable/awesome-reMarkable#launchers)
[![Mentioned in Awesome reMarkable](https://awesome.re/mentioned-badge.svg)](https://github.com/reHackable/awesome-reMarkable)

<img src="icon.png" width="25%">

Implementation of rust tetris_core on the reMarkable using libremarkable

<img width="50%" src="https://transfer.cosmos-ink.net/AQWDL/192.168.2.93.jpg">

The patterns are some totally random functions. If someone wants to do something better (not just functions) or just loves math, [go ahead](https://github.com/LinusCDE/retris/blob/929a597acbb9215dcb53663a4a9a415fb7bbc175/src/scene/game_scene.rs#L50).

## Controlling

- Move Left and Right: Hardware and Software buttons or swipe left and right
- Move down: Swipe down
- Rotate: Middle hardware button or tap anywhere

## Installation

### Prebuilt binary/program

- Go the the [releases page](https://github.com/LinusCDE/retris/releases)
- Get the newest released "retris" file and copy it onto your remarkable, using e.g. FileZilla, WinSCP or scp.
- SSH into your remarkable and mark the file as executable with `chmod +x retris`
- Stop xochitl (the interface) with `systemctl stop xochitl`
- Start the game with `./retris`
- After you're done, restart xochitl with `systemctl start xochitl`

### Compiling

In general building should work on most toolchains. You generally wanna target armv7-unknown-linux-gnueabihf for any remarkable.
But as with all things in life, stuff never works great on every setup.

That's why I recommend to nowadays build with the rust image from [toltec-dev/toolchain](https://github.com/toltec-dev/toolchain). It is the most modern and the closest to the actual reMarkable system as you're gonna get as of now.

To make it easier to use, I found that you can use the rust image (`ghcr.io/toltec-dev/rust:v3.2`, [all versions](https://github.com/toltec-dev/toolchain/pkgs/container/rust)).
This is done using the `Cross.toml` file. So you should just need to run `cross build --target=armv7-unknown-linux-gnueabihf --release` and it will use the above image (or possibly newer if this readme gets out-of-date).

## reMarkable 2 support

This app cant actually drive the rM 2 framebuffer. It needs [rm2fb](https://github.com/ddvk/remarkable2-framebuffer/) for that.

If you execute retris from ssh, be sure to have followed rm2fb steps to enable the support. When launching through a launcher (from toltec) it should just work but have more ghosting on the rM2.
