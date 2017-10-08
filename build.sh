#!/usr/bin/env bash -e
. $HOME/.cargo/env

cd frontend
CARGO_INCREMENTAL=1 cargo build --target armv7-unknown-linux-gnueabihf --release
