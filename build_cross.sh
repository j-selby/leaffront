#!/bin/bash
. $HOME/.cargo/env

set -e

CARGO_INCREMENTAL=1 PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --features raspberry_pi
strip target/release/leaffront-station --strip-unneeded
cargo deb --no-build --no-strip

cd dbus
CARGO_INCREMENTAL=1 PKG_CONFIG_ALLOW_CROSS=1 cargo build --release
strip ../target/release/leaffront-dbus --strip-unneeded
cargo deb --no-build --no-strip
