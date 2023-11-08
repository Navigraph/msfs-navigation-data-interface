run `rustup toolchain install 1.65`, `rustup target add wasm32-wasi`

building: `.\build.bat`

you need clang v15 to compile this properly. you MUST build with the release flag or the sim crashes
you need wasi sdk v19 and set a WASI_SDK environment variable to its install location