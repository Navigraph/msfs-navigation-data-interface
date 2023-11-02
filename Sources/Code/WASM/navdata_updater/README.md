run `rustup toolchain install 1.65`, `rustup target add wasm32-wasi`

building: `cargo build --release; .\copy.bat`

you need clang v15 to compile this properly. you MUST build with the release flag or the sim crashes