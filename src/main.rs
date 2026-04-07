mod lexer;
mod emitter;
mod wav;

use std::{env, fs};
use std::process;
use crate::emitter::Emitter;
use crate::lexer::{Lexer, Token};

fn main() {
    // collect command line arguments
    let args: Vec<String> = env::args().collect();

    // check if we have enough arguments
    // expected: mus-asm <input.mus> [-o <output.wav>]
    if args.len() < 2 {
        println!("Usage: mus-asm <input.mus> [-o <output.wav>]");
        process::exit(1);
    }

    let input_path = &args[1];
    let mut output_path = String::from("output.wav");

    // simple argument parsing
    let mut i = 2;
    while i < args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            output_path = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    // start the translation process
    if let Err(e) = run_translation(input_path, &output_path) {
        eprintln!("[ERROR] Translation error: {}", e);
        process::exit(1);
    }

    println!("[DEBUG] Successfully translated {} to {}", input_path, output_path);
}

fn run_translation(input: &str, output: &str) -> std::io::Result<()> {
    println!("[DEBUG] Reading file...");
    let source = fs::read_to_string(input)?;

    println!("[DEBUG] Lexing...");
    let mut lexer = Lexer::new(&source);
    let mut tokens = Vec::new();

    while let token = lexer.next_token() {
        if token == Token::EOF { break; }
        tokens.push(token);
        // failsafe: if you have 10M tokens for a tiny file, something is wrong
        if tokens.len() > 1_000_000 {
            panic!("[ERROR] Too many tokens! Lexer is broken.");
        }
    }

    println!("[DEBUG] Starting emitter with {} tokens...", tokens.len());
    let mut emitter = Emitter::new();
    let total_samples = emitter.translate(&tokens);

    // finalize and stitch chunks from .cache
    if total_samples > 0 {
        wav::finalize_wav(output, total_samples, emitter.chunk_count)?;
    } else {
        println!("[WARN] No samples were generated");
    }

    Ok(())
}