# file2wav

A tool to insert any file into a WAV audio file. It works using a kind of a steganography technique consisting of modify the less significant bits of the WAV audio samples.

If the audio file doesn't have enought duration to insert all the file bits it will loop until finish.

## Build

Requires Rust installed in your system. Then just run:

```bash
cargo build
```

## Usage

```bash
file2wav [OPTIONS] <input_wav> <file_path> <output_wav> [SUBCOMMAND]
```

Additional options:

- --bits-per-sample: How many bits of the original file are inserted in the LSB (Less significant bits) of the audio samples. Max 8. 2 by default.
- --skip: Insert each *skip*  samples. 0 by default.

### Examples

#### Basic Insert

```bash
file2wav insert -a audio/kauwela.wav -f test_bici.jpg -o kauwela_inserted.wav
```

#### Insert modifying bits per sample and skip

You can use the **bits per sample** option and **skip** option to increase or decrease the rate of information inserted.

For instance with a modification of one bit per sample and skipping one audio sample the sound modification will be even less noticeable.

```bash
file2wav insert -a audio/kauwela.wav -f test_bici.jpg -o kauwela_inserted.wav \
 --bits-per-sample 1 --skip 1
```

## Credits

Author: Alberto López Sánchez
Audio: Kauwela - Scandinavianz & Limujii [soundcloud](https://soundcloud.com/scandinavianz/scandinavianz-limujii-kauwela)
