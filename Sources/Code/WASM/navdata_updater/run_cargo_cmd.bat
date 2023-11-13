@echo off

set "CLANG_LIB_WASI=%WASI_SDK%\\lib\\clang\\15.0.7\\lib\\wasi"
set "WASI_SYSROOT=%WASI_SDK%\\share\\wasi-sysroot"
set "CC=%WASI_SDK%\\bin\\clang --sysroot=%WASI_SYSROOT%"
set "AR=%WASI_SDK%\\bin\\llvm-ar"
set "CC_wasm32_wasi=%WASI_SDK%\\bin\\clang"
set "LIBSQLITE3_FLAGS=-DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_SHARED_CACHE -D_LARGEFILE64_SOURCE"
set "RUSTFLAGS=-Clink-arg=-L%CLANG_LIB_WASI% -Clink-arg=-lclang_rt.builtins-wasm32 -Clink-arg=--export-table -Clink-arg=--export=malloc -Clink-arg=--export=free"


:: Run the Cargo command passed as an argument
if "%1"=="" (
    echo No Cargo command specified.
) else (
    cargo %*
)