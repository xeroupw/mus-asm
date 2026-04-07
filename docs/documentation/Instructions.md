# Full Instruction Set

Detailed list of all available commands in the mus-asm virtual machine.

## Memory & Arithmetic
- `mov <reg>, <val>`: Set register [r0-r7] to a specific floating-point value.
- `add <reg>, <val>`: Increment or decrement (using negative values) register [r0-r7].

## Flow Control
- `label:`: Define a jump target in the code.
- `jmp <label>`: Unconditional jump to the specified label.
- `jnz <label>`: Jump to label if the `zero_flag` is false (occurs when `r1 > 0`).

## Audio & DSP
- `voice <n>`: Switch active voice context [0-7]. Each voice has its own registers and state.
- `wave <type>`: Set oscillator type: `sine`, `square`, `tri`, `saw`, or `noise`.
- `note_on`: Trigger the ADSR envelope (starts Attack stage).
- `note_off`: Trigger the ADSR Release stage.
- `delay <t> <f> <m>`: Set global ring-buffer delay (time in samples, feedback 0-1, mix 0-1).

## Generation & Timing
- `out`: Generate exactly one audio sample and advance internal phases.
- `fill <n>`: Generate `<n>` samples sequentially. At 44.1kHz, `fill 44100` is 1 second.