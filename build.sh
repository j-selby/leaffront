#!/usr/bin/env bash -e
. $HOME/.cargo/env

CARGO_INCREMENTAL=1 cargo build --target armv7-unknown-linux-gnueabihf -vv --release
