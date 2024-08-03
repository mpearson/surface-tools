# Surface Tools

This repo is an experimental sandbox for 3D mesh manipulation and simulation in the browser.
Algorithm demos will be implemented in React/Typescript and THREE.js, built using vite.js.
Algorithms and data structures will be implemented in rust and compiled to WASM.


# Goals

[ ] Figure out how to do Rust -> webassembly
[ ] Port the Half-Edge data structure from https://github.com/mpearson/surface-tools-unity


# Building for Windows
```
sudo apt install -y gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
```

# Running in the browser
```
# cargo install wasm-server-runner

cargo install wasm-bindgen-cli
```
