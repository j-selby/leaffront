#!/bin/bash
. $HOME/.cargo/env

CARGO_INCREMENTAL=1 cargo build --target armv7-unknown-linux-gnueabihf --release --features raspberry_pi
arm-linux-gnueabihf-strip target/armv7-unknown-linux-gnueabihf/release/leaffront --strip-unneeded
cargo deb --target armv7-unknown-linux-gnueabihf --no-build --no-strip
