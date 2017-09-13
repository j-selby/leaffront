#!/usr/bin/env bash
source $HOME/.cargo/env

set -e

cargo build --target armv7-unknown-linux-gnueabihf -vv --release
