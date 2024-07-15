#!/bin/sh
cargo build --target x86_64-pc-windows-gnu &&
cp ../../target/x86_64-pc-windows-gnu/debug/bevy_experiment.exe /mnt/d/
cp -r ../../target/x86_64-pc-windows-gnu /mnt/d/
# exec ./bevy_experiment.exe "$@"
