use std::fs;
use std::io::{Result, Write};
use std::path::PathBuf;

pub fn substr(s: &str, start: usize, len: usize) -> String {
    s.chars().skip(start).take(len).collect()
}

pub fn save_file(content: &[u8], dest_path: &PathBuf) -> Result<()> {
    let mut file_out = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dest_path)?;
    file_out.write_all(content)?;
    file_out.flush()?;
    Ok(())
}
