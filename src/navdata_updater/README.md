# Building:

- Install Docker
- Run `.\build.bat` to build, and `.\run_cargo_cmd.bat` followed by a Cargo command to run a specified command (e.g `.\run_cargo_cmd.bat clippy`)


# Warning
The only file system functions that properly work are `fs::remove_file`, `fs::File::create`, `fs::create_dir`, `fs::remove_dir`, `fs::remove_file`. All the other functions regarding the filesystem do not work due to the MSFS implementation of WASI