#!/usr/bin/env bash

export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
export RUST_BACKTRACE=1

cargo run --release --bin simula
