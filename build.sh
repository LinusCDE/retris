#!/bin/sh

cd "`dirname \"$0\"`"

if ! which rustup >/dev/null 2>&1; then
  echo 'Please install rustup either from your system package manager or https://rustup.rs/' >&2
  exit 1
fi

# Not 100% sure if needed. Might get away without
#if ! rustup show active-toolchain | grep "nightly-" >/dev/null 2>&1; then
#  echo 'Please use the nightly build.' >&2
#  echo 'Run "rustup install nightly; rustup default nightly" to do so' >&2
#  exit 1
#fi

if ! rustup target list | grep "armv7-unknown-linux-gnueabihf (installed)" >/dev/null 2>&1; then
  echo 'You need to add the armv7-unknown-linux-gnueabihf target' >&2
  echo 'Run "rustup target add armv7-unknown-linux-gnueabihf" to do so' >&2
  exit 1
fi

if [ ! -d /usr/local/oecore-x86_64/ ]; then
  echo "Couldn't find the oecore toolchain at its default location" >&2
  echo "Please install it from https://remarkable.engineering/" >&2
  exit 1
fi

# Build (--target=... is already set in .config/cargo)
cargo build --release

# Reduce file size siginificantly
source /usr/local/oecore-x86_64/environment-setup-cortexa9hf-neon-oe-linux-gnueabi
$STRIP target/armv7-unknown-linux-gnueabihf/release/retris
