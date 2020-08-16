# retris
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

- Make sure to have rustup and a current toolchain (nightly might be needed)
- Install the [oecore toolchain](https://remarkable.engineering/).
  - If you're not using linux, you might want to adjust the path in `.config/cargo`
- Compile it with `cargo build --release`. It should automatically cross-compile.
