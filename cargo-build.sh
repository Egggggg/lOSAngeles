#! /bin/sh

cd kernel
cargo build --target x86_64-angeles.json
cd ..