## Music Assembler
Licensed under the Apache License, Version 2.0. See LICENSE for details.<BR><Br>
is a low-level audio synthesis engine and virtual machine written in Rust.<br>
It compiles custom assembly-like scripts into high-fidelity audio using multi-channel mixing, ADSR envelopes, and global effects. Designed for high performance and granular control over digital sound design.
<br>

### Features:
- Fast processing: Utilizes a "chunk-based" system that significantly accelerates WAV file assembly.
- Low-level control: Provides direct access to registers and oscillators for granular sound design.
- Extensible architecture: Modular design makes it easy to implement and add new DSP effects.

### Introduction:
See [documentation](https://xeroup.github.io/mus-asm/) for getting started on mus-asm

### Installation:
Download mus-asm from [releases](https://github.com/xeroup/mus-asm/releases).<br>
Place your `.mus` file next to the executable and run it using the command:

```bash
# macOS/Linux
mus-asm <file.mus> [-o <file.wav>]

# windows
./mus-asm <file.mus> [-o <file.wav>]
```

