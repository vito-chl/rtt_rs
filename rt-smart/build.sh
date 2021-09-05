#!/bin/bash
cargo build --release --target rt_smart.json -Z build-std=core,alloc,libc