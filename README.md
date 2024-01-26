# pphd8extract
A simple program written in rust to extract VAG files from PPHD8 files, a tar-like format used in old Play Station Consoles to store audio files.
<p align="center">
 <img src="https://github.com/LDiazN/pphd8extract/raw/main/img/pphd8extract.jpg" alt="Watch the gameplay video in YouTube" border="10" />
</p>

This is a simple program I made as a request from an acquaintance. The problem seemed interesting so I downloaded [ImHex](https://github.com/WerWolv/ImHex) and started reverse engineering the PPHD8 files. This was my first time reverse engineering a file format and it was a lot of fun, at the start it was super intimidating but after starting to see some patterns here and there I was able to extract a playable version of the VAG files. 

The code is written in **Rust** since it's a language that i'm interested in learning and it seems very fit for this kind of task. I used [Rayon](https://crates.io/crates/rayon) for parallel execution when processing and saving multiple files, and [egui](https://github.com/emilk/egui) for UI (but I also provide a CLI version if you prefer). 

I plan to write a complete article about this program, but for now check my previous [blog posts](https://ldiazn.github.io/blog).

To actually play a `.VAG` file, you can use [MFAudio](https://www.zophar.net/utilities/ps2util/mfaudio-1-1.html).

# Installation

I provide the executable files for windows, but you can also build it from scratch having [Rust installed](https://www.rust-lang.org/tools/install).

1. Clone this repository
2. Move to the `pphd8extract` directory
3. Compile the executable file you want:
   1. **GUI Executable**: `cargo build --release --bin gui`
   2. **Command line executable**: `cargo build --release --bin cli`
4. In any case, the output executable will be in `target/release`, you can rename them as you want

Note that I have only tested this program on Windows, it might not work in Mac or Linux.

# Usage
## UI
1. Drop the files you want to process within the program window.
2. Select an output directory.
3. Click `extract`. If there is no errors, your VAG files should be in the location you requested.

## CLI
call the executable file with the path to the pphd8 file as first argument, and the output dir as the second argument:

```powershell
./cli.exe D:/path/to/file.pphd8 D:/path/to/output_dir/
```

