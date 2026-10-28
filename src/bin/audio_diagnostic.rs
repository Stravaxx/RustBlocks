use std::{env, path::Path};

fn main() {
    let mut args = env::args().skip(1);
    let path = args
        .next()
        .unwrap_or_else(|| "rustblocks-bytebeat.wav".to_owned());
    let seconds = args.next().and_then(|v| v.parse().ok()).unwrap_or(10);
    let rate = args
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(rustblocks::audio::bytebeat::DEFAULT_SAMPLE_RATE);
    if seconds == 0 || seconds > 600 || !(8_000..=192_000).contains(&rate) {
        eprintln!(
            "Usage: rustblocks-audio [output.wav] [seconds: 1..600] [sample-rate: 8000..192000]"
        );
        std::process::exit(2);
    }
    if let Err(error) =
        rustblocks::audio::wav::write_diagnostic_wav(Path::new(&path), seconds, rate)
    {
        eprintln!("Could not write WAV file: {error}");
        std::process::exit(1);
    }
    println!("Wrote {path} ({seconds}s, {rate} Hz, mono PCM16)");
}
