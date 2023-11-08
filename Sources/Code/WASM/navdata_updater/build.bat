@echo off

set "CLANG_LIB_WASI=%WASI_SDK%\\lib\\clang\\15.0.7\\lib\\wasi"
set "WASI_SYSROOT=%WASI_SDK%\\share\\wasi-sysroot"
set "CC=%WASI_SDK%\\bin\\clang --sysroot=%WASI_SYSROOT%"
set "AR=%WASI_SDK%\\bin\\llvm-ar"
set "CC_wasm32_wasi=%WASI_SDK%\\bin\\clang"
set "LIBSQLITE3_FLAGS=-DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_SHARED_CACHE -D_LARGEFILE64_SOURCE"
set "RUSTFLAGS=-Clink-arg=--export-table -Clink-arg=--export=malloc -Clink-arg=--export=free -Clink-arg=-L%CLANG_LIB_WASI% -Clink-arg=-lclang_rt.builtins-wasm32"


cargo build --release
copy "target\wasm32-wasi\release\navdata_updater.wasm" "..\..\..\..\PackageSources\SimObjects\Airplanes\Navigraph_Navdata_Updater_Aircraft\panel\navdata_updater.wasm"