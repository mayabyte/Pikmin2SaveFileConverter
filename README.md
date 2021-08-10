# Pikmin 2 Save File Converter

Converts Pikmin 2 save files in `.gci` format between regions.

Linux/MacOS usage:
```bash
$ ./p2saveconvert_linux YOUR_SAVE_FILE.gci --region JP
```

Windows usage:
```powershell
> .\p2saveconvert_windows.exe '.\YOUR_SAVE_FILE.gci' -r JP
```

Use the `--help` or `-h` argument to see detailed usage instructions and additional arguments.

# Downloads
tbd

# Developing
`p2saveconvert` is written in Rust and can be compiled like any other Rust project: `cargo build --release`. Alternatively you can run `package.sh` to build binaries for Linux, MacOS, and Windows in the `./package` folder provided you either have Docker or Podman installed.

If you're unfamiliar with Rust, you can install `cargo` (Rust's build tool) using Rustup: https://rustup.rs/.

## Support
Ping me on Discord (chemical#7290) if you have any questions about the tool :)
