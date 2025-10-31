#!/bin/bash
set -e

. $HOME/.cargo/env

CARGO_INCREMENTAL=1 PKG_CONFIG_ALLOW_CROSS=1 cargo build --target armv7-unknown-linux-gnueabihf --release --features glutin
arm-linux-gnueabihf-strip target/armv7-unknown-linux-gnueabihf/release/leaffront-station --strip-unneeded
# cargo deb --target armv7-unknown-linux-gnueabihf --no-build --no-strip
