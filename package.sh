#!/bin/bash

cargo install cross

cross build --release --target=x86_64-apple-darwin
cross build --release --target=x86_64-pc-windows-gnu
cross build --release --target=x86_64-unknown-linux-gnu

rm -rf ./package
mkdir ./package
mv ./target/x86_64-apple-darwin/release/p2saveconvert ./package/p2saveconvert_macos
mv ./target/x86_64-pc-windows-gnu/release/p2saveconvert.exe ./package/p2saveconvert_windows.exe
mv ./target/x86_64-unknown-linux-gnu/release/p2saveconvert ./package/p2saveconvert_linux

cargo clean --release --target=x86_64-apple-darwin
cargo clean --release --target=x86_64-pc-windows-gnu
cargo clean --release --target=x86_64-unknown-linux-gnu
