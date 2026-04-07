# ADSR Envelope

Sound dynamics are shaped using a standard Attack-Decay-Sustain-Release envelope. Values in `r4`-`r7` determine the speed of transitions.

## Usage Logic
1. Initialize values in `r4`, `r5`, `r6`, and `r7`.
2. Use `note_on` to start the sound.
3. Use `fill` to let the sound play.
4. Use `note_off` to begin the release fade-out.

```asm
; Soft synth example
mov r4, 0.005  ; Slow attack
mov r5, 0.01   ; Fast decay
mov r6, 0.3    ; 30% sustain volume
mov r7, 0.001  ; Long release fade

note_on
fill 40000     ; Sustain the note
note_off
fill 10000     ; Hear the release tail
```