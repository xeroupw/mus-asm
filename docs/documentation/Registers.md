# Register Mapping

Each of the 8 voices in **mus-asm** has its own set of 8 registers (`r0`-`r7`). These control everything from pitch to envelope shapes.

| Register | Name        | Description                                                                 |
|----------|-------------|-----------------------------------------------------------------------------|
| `r0`     | **FREQ**    | Oscillator frequency in Hz. Can be changed during `fill` for pitch slides.  |
| `r1`     | **AMP**     | Master volume for the voice (0.0 - 1.0). Updates the `zero_flag`.           |
| `r2`     | **PAN**     | Stereo panning balance (currently reserved).                                |
| `r3`     | **MOD**     | Modulation depth or waveform-specific parameter (currently reserved).       |
| `r4`     | **ATTACK**  | ADSR: Amount added to amplitude per sample until 1.0 is reached.            |
| `r5`     | **DECAY**   | ADSR: Amount subtracted per sample until reaching the Sustain level.        |
| `r6`     | **SUSTAIN** | ADSR: The target amplitude level (0.0 - 1.0) held during the sustain stage. |
| `r7`     | **RELEASE** | ADSR: Amount subtracted per sample after `note_off` is called.              |

> **Note:** The `zero_flag` used by `jnz` is strictly tied to `r1`. If `r1` is greater than 0, the flag is "False" (non-zero).