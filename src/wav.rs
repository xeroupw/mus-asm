use std::fs::{self, File};
use std::io::{BufWriter, Read, Write, Result};

pub fn finalize_wav(path: &str, total_samples: usize, chunk_count: usize) -> Result<()> {
    let output_file = File::create(path)?;
    let mut writer = BufWriter::new(output_file);

    let data_size = (total_samples * 2) as u32;
    let sample_rate: u32 = 44100;
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;

    // 1. riff header
    writer.write_all(b"RIFF")?;
    writer.write_all(&(36 + data_size).to_le_bytes())?;
    writer.write_all(b"WAVE")?;

    // 2. fmt sub-chunk
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?; // chunk size
    writer.write_all(&1u16.to_le_bytes())?;  // pcm format
    writer.write_all(&channels.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&(sample_rate * channels as u32 * bits_per_sample as u32 / 8).to_le_bytes())?; // byte rate
    writer.write_all(&(channels * bits_per_sample / 8).to_le_bytes())?; // block align
    writer.write_all(&bits_per_sample.to_le_bytes())?;

    // 3. data sub-chunk
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    // 4. stitch chunks from disk
    println!("[INFO] Stitching {} chunks...", chunk_count);
    for i in 0..chunk_count {
        let chunk_path = format!(".cache/chunk_{}.raw", i);
        let mut chunk_file = File::open(&chunk_path)?;

        // buffer for reading chunk (keeps ram usage low)
        let mut chunk_data = Vec::new();
        chunk_file.read_to_end(&mut chunk_data)?;
        writer.write_all(&chunk_data)?;
    }

    // flush everything to disk before cleaning up
    writer.flush()?;

    // 5. cleanup
    fs::remove_dir_all(".cache")?;
    println!("[DEBUG] Finalized: {} ({} samples)", path, total_samples);

    Ok(())
}