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
See https://github.com/mayabyte/Pikmin2SaveFileConverter/releases for downloads of pre-built binaries for the 3 major platforms.

If you're on a platform for which there's no pre-compiled binary, see the next section for how to build the tool yourself.

# Developing
`p2saveconvert` is written in Rust and can be compiled like any other Rust project: `cargo build --release`. Alternatively you can run `package.sh` to build binaries for Linux, MacOS, and Windows in the `./package` folder provided you either have Docker or Podman installed.

If you're unfamiliar with Rust, you can install `cargo` (Rust's build tool) using Rustup: https://rustup.rs/.

## Support
Ping me on Discord (chemical#7290) if you have any questions about the tool :)
