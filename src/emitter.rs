use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use crate::lexer::Token;

const CHUNK_SIZE: usize = 8192;
const MAX_DELAY_SAMPLES: usize = 44100 * 2; // max 2 seconds delay

#[derive(Clone, Copy, PartialEq)]
enum Waveform {
    Sine, Square, Triangle, Sawtooth, Noise,
}

#[derive(Clone, Copy, PartialEq)]
enum AdsrStage {
    Idle, Attack, Decay, Sustain, Release,
}

struct Voice {
    registers: [f32; 8],
    phase: f32,
    wave: Waveform,
    zero_flag: bool,
    adsr_stage: AdsrStage,
    current_amp: f32,
}

impl Voice {
    fn new() -> Self {
        Self {
            registers: [0.0; 8],
            phase: 0.0,
            wave: Waveform::Sine,
            zero_flag: false,
            adsr_stage: AdsrStage::Idle,
            current_amp: 0.0,
        }
    }

    fn update_adsr(&mut self) {
        match self.adsr_stage {
            AdsrStage::Attack => {
                self.current_amp += self.registers[4];
                if self.current_amp >= 1.0 {
                    self.current_amp = 1.0;
                    self.adsr_stage = AdsrStage::Decay;
                }
            },
            AdsrStage::Decay => {
                self.current_amp -= self.registers[5];
                if self.current_amp <= self.registers[6] {
                    self.current_amp = self.registers[6];
                    self.adsr_stage = AdsrStage::Sustain;
                }
            },
            AdsrStage::Release => {
                self.current_amp -= self.registers[7];
                if self.current_amp <= 0.0 {
                    self.current_amp = 0.0;
                    self.adsr_stage = AdsrStage::Idle;
                }
            },
            _ => {}
        }
    }
}

pub struct Emitter {
    voices: Vec<Voice>,
    current_voice: usize,
    sample_rate: u32,
    labels: HashMap<String, usize>,
    pub chunk_count: usize,
    rng_state: u32,

    // delay engine
    delay_buffer: Vec<f32>,
    delay_ptr: usize,
    delay_time: usize,    // in samples
    delay_feedback: f32, // 0.0 to 1.0
    delay_mix: f32,      // 0.0 to 1.0
}

impl Emitter {
    pub fn new() -> Self {
        if fs::metadata(".cache").is_ok() {
            let _ = fs::remove_dir_all(".cache");
        }
        fs::create_dir(".cache").expect("failed to create cache");

        let mut voices = Vec::new();
        for _ in 0..8 { voices.push(Voice::new()); }

        Self {
            voices,
            current_voice: 0,
            sample_rate: 44100,
            labels: HashMap::new(),
            chunk_count: 0,
            rng_state: 12345,
            delay_buffer: vec![0.0; MAX_DELAY_SAMPLES],
            delay_ptr: 0,
            delay_time: 0,
            delay_feedback: 0.5,
            delay_mix: 0.0,
        }
    }

    fn next_random(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        self.rng_state &= 0x7FFFFFFF;
        (self.rng_state as f32 / 0x7FFFFFFF as f32) * 2.0 - 1.0
    }

    fn apply_delay(&mut self, dry_sample: f32) -> f32 {
        if self.delay_time == 0 || self.delay_mix == 0.0 {
            return dry_sample;
        }

        // calculate position of the delayed sample
        let read_ptr = (self.delay_ptr + MAX_DELAY_SAMPLES - self.delay_time) % MAX_DELAY_SAMPLES;
        let delayed_sample = self.delay_buffer[read_ptr];

        // write to buffer: input + feedback from previous delay
        self.delay_buffer[self.delay_ptr] = dry_sample + (delayed_sample * self.delay_feedback);

        // advance pointer
        self.delay_ptr = (self.delay_ptr + 1) % MAX_DELAY_SAMPLES;

        // mix dry and wet
        dry_sample * (1.0 - self.delay_mix) + delayed_sample * self.delay_mix
    }

    fn generate_mixed_sample(&mut self) -> i16 {
        let mut dry_mixed: f32 = 0.0;
        let mut active_count = 0;

        for i in 0..8 {
            let voice = &mut self.voices[i];
            voice.update_adsr();

            let amp = voice.registers[1] * voice.current_amp;
            if amp <= 0.0 && voice.adsr_stage == AdsrStage::Idle { continue; }
            active_count += 1;

            let val = match voice.wave {
                Waveform::Sine => voice.phase.sin(),
                Waveform::Square => if voice.phase < std::f32::consts::PI { 1.0 } else { -1.0 },
                Waveform::Triangle => {
                    let p = voice.phase / (2.0 * std::f32::consts::PI);
                    if p < 0.5 { 4.0 * p - 1.0 } else { 3.0 - 4.0 * p }
                },
                Waveform::Sawtooth => (voice.phase / std::f32::consts::PI) - 1.0,
                Waveform::Noise => self.next_random(),
            };
            dry_mixed += val * amp;
        }

        if active_count > 1 {
            dry_mixed *= 1.0 / (active_count as f32).sqrt();
        }

        // process through global delay
        let final_sample = self.apply_delay(dry_mixed);

        (final_sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
    }

    fn flush_to_cache(&mut self, buffer: &mut Vec<i16>) {
        if buffer.is_empty() { return; }
        let path = format!(".cache/chunk_{}.raw", self.chunk_count);
        let file = File::create(&path).expect("failed to create chunk");
        let mut writer = BufWriter::new(file);
        for &sample in buffer.iter() {
            writer.write_all(&sample.to_le_bytes()).expect("write error");
        }
        writer.flush().expect("flush error");
        buffer.clear();
        self.chunk_count += 1;
    }

    pub fn translate(&mut self, tokens: &[Token]) -> usize {
        for (idx, token) in tokens.iter().enumerate() {
            if let Token::Label(name) = token { self.labels.insert(name.clone(), idx); }
        }

        let mut buffer = Vec::with_capacity(CHUNK_SIZE + 1);
        let mut i = 0;
        let mut total_samples: usize = 0;

        while i < tokens.len() {
            let mut stepped = false;
            match &tokens[i] {
                Token::Instruction(name) => {
                    match name.as_str() {
                        "delay" => {
                            // delay [time_samples] [feedback] [mix]
                            if let (Some(Token::Number(t)), Some(Token::Number(f)), Some(Token::Number(m))) =
                                (tokens.get(i+1), tokens.get(i+2), tokens.get(i+3)) {
                                self.delay_time = (*t as usize).min(MAX_DELAY_SAMPLES - 1);
                                self.delay_feedback = *f;
                                self.delay_mix = *m;
                                i += 4; stepped = true;
                            }
                        },
                        "voice" => {
                            if let Some(Token::Number(v_idx)) = tokens.get(i+1) {
                                self.current_voice = (*v_idx as usize).min(7);
                                i += 2; stepped = true;
                            }
                        },
                        "wave" => {
                            if let Some(Token::Instruction(w)) = tokens.get(i+1) {
                                self.voices[self.current_voice].wave = match w.as_str() {
                                    "sine" => Waveform::Sine, "square" => Waveform::Square,
                                    "tri" => Waveform::Triangle, "saw" => Waveform::Sawtooth,
                                    "noise" => Waveform::Noise, _ => Waveform::Sine,
                                };
                                i += 2; stepped = true;
                            }
                        },
                        "mov" => {
                            if let (Some(Token::Register(r)), Some(Token::Number(v))) = (tokens.get(i+1), tokens.get(i+3)) {
                                let idx = self.parse_reg(r);
                                self.voices[self.current_voice].registers[idx] = *v;
                                if idx == 1 { self.voices[self.current_voice].zero_flag = *v <= 0.0; }
                                i += 4; stepped = true;
                            }
                        },
                        "add" => {
                            if let (Some(Token::Register(r)), Some(Token::Number(v))) = (tokens.get(i+1), tokens.get(i+3)) {
                                let idx = self.parse_reg(r);
                                self.voices[self.current_voice].registers[idx] += *v;
                                if idx == 1 { self.voices[self.current_voice].zero_flag = self.voices[self.current_voice].registers[idx] <= 0.0; }
                                i += 4; stepped = true;
                            }
                        },
                        "note_on" => { self.voices[self.current_voice].adsr_stage = AdsrStage::Attack; i += 1; stepped = true; },
                        "note_off" => { self.voices[self.current_voice].adsr_stage = AdsrStage::Release; i += 1; stepped = true; },
                        "out" => {
                            buffer.push(self.generate_mixed_sample());
                            total_samples += 1;
                            self.update_all_phases();
                            if buffer.len() >= CHUNK_SIZE { self.flush_to_cache(&mut buffer); }
                            i += 1; stepped = true;
                        },
                        "fill" => {
                            if let Some(Token::Number(c)) = tokens.get(i+1) {
                                for _ in 0..(*c as usize) {
                                    buffer.push(self.generate_mixed_sample());
                                    total_samples += 1;
                                    self.update_all_phases();
                                    if buffer.len() >= CHUNK_SIZE { self.flush_to_cache(&mut buffer); }
                                }
                                i += 2; stepped = true;
                            }
                        },
                        "jmp" => { if let Some(Token::Instruction(l)) = tokens.get(i+1) { i = *self.labels.get(l).expect("lbl"); stepped = true; } },
                        "jnz" => { if let Some(Token::Instruction(l)) = tokens.get(i+1) { if !self.voices[self.current_voice].zero_flag { i = *self.labels.get(l).expect("lbl"); } else { i += 2; } stepped = true; } },
                        _ => {}
                    }
                },
                _ => {}
            }
            if !stepped { i += 1; }
        }
        self.flush_to_cache(&mut buffer);
        total_samples
    }

    fn update_all_phases(&mut self) {
        for v in self.voices.iter_mut() {
            v.phase += 2.0 * std::f32::consts::PI * v.registers[0] / self.sample_rate as f32;
            if v.phase > 2.0 * std::f32::consts::PI { v.phase -= 2.0 * std::f32::consts::PI; }
        }
    }

    fn parse_reg(&self, name: &str) -> usize { name[1..].parse::<usize>().unwrap_or(0).min(7) }
}