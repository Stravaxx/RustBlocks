use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

pub fn write_diagnostic_wav(path: &Path, seconds: u32, sample_rate: u32) -> io::Result<()> {
    let sample_count = seconds.saturating_mul(sample_rate);
    let data_bytes = sample_count.saturating_mul(2);
    let mut file = File::create(path)?;
    file.write_all(b"RIFF")?;
    file.write_all(&(36_u32.saturating_add(data_bytes)).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16_u32.to_le_bytes())?;
    file.write_all(&1_u16.to_le_bytes())?;
    file.write_all(&1_u16.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&sample_rate.saturating_mul(2).to_le_bytes())?;
    file.write_all(&2_u16.to_le_bytes())?;
    file.write_all(&16_u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    let mut buffer = Vec::with_capacity(16_384);
    let mut index = 0_u32;
    while index < sample_count {
        buffer.clear();
        let block_end = index.saturating_add(8_192).min(sample_count);
        while index < block_end {
            let sample = super::bytebeat::sample_at(u64::from(index), sample_rate);
            let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
            buffer.extend_from_slice(&pcm.to_le_bytes());
            index += 1;
        }
        file.write_all(&buffer)?;
    }
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_valid_pcm_header_and_sample_data() {
        let path =
            std::env::temp_dir().join(format!("rustblocks-audio-{}.wav", std::process::id()));
        write_diagnostic_wav(&path, 1, 8_000).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(bytes.len(), 44 + 16_000);
        std::fs::remove_file(path).unwrap();
    }
}
