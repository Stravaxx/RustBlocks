pub mod bytebeat;
pub mod wav;

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

#[derive(Clone)]
pub struct AudioControls {
    pub music_enabled: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub music_volume_bits: Arc<AtomicU32>,
    pub effects_enabled: Arc<AtomicBool>,
    pub effects_volume_bits: Arc<AtomicU32>,
    effect_trigger: Arc<AtomicU32>,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum SoundEffect {
    Move = 1,
    Rotate = 2,
    Drop = 3,
    Lock = 4,
    Line = 5,
    Multi = 6,
    Combo = 7,
    Level = 8,
    GameOver = 9,
    Menu = 10,
}

impl Default for AudioControls {
    fn default() -> Self {
        Self {
            music_enabled: Arc::new(AtomicBool::new(true)),
            paused: Arc::new(AtomicBool::new(false)),
            music_volume_bits: Arc::new(AtomicU32::new(0.18_f32.to_bits())),
            effects_enabled: Arc::new(AtomicBool::new(true)),
            effects_volume_bits: Arc::new(AtomicU32::new(0.7_f32.to_bits())),
            effect_trigger: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl AudioControls {
    pub fn set_volume(&self, value: f32) {
        self.music_volume_bits
            .store(safe_volume(value).to_bits(), Ordering::Relaxed);
    }

    pub fn set_effects_volume(&self, value: f32) {
        self.effects_volume_bits
            .store(safe_volume(value).to_bits(), Ordering::Relaxed);
    }

    pub fn trigger(&self, effect: SoundEffect) {
        self.effect_trigger.store(effect as u32, Ordering::Relaxed);
    }
}

fn safe_volume(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub struct AudioOutput {
    _stream: cpal::Stream,
}

impl AudioOutput {
    pub fn start(controls: AudioControls) -> Result<Self, String> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No default audio output device")?;
        let config = device
            .default_output_config()
            .map_err(|e| format!("Audio device configuration: {e}"))?;
        let sample_rate = config.sample_rate().0;
        let channels = usize::from(config.channels());
        let stream_config: cpal::StreamConfig = config.clone().into();
        let error_controls = controls.clone();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                make_callback::<f32>(sample_rate, channels, controls),
                move |e| {
                    eprintln!("Audio output error: {e}");
                    error_controls.music_enabled.store(false, Ordering::Relaxed);
                },
                None,
            ),
            cpal::SampleFormat::I16 => device.build_output_stream(
                &stream_config,
                make_callback::<i16>(sample_rate, channels, controls),
                move |e| {
                    eprintln!("Audio output error: {e}");
                    error_controls.music_enabled.store(false, Ordering::Relaxed);
                },
                None,
            ),
            cpal::SampleFormat::U16 => device.build_output_stream(
                &stream_config,
                make_callback::<u16>(sample_rate, channels, controls),
                move |e| {
                    eprintln!("Audio output error: {e}");
                    error_controls.music_enabled.store(false, Ordering::Relaxed);
                },
                None,
            ),
            format => return Err(format!("Unsupported audio sample format: {format:?}")),
        }
        .map_err(|e| format!("Could not create audio stream: {e}"))?;
        stream
            .play()
            .map_err(|e| format!("Could not start audio stream: {e}"))?;
        Ok(Self { _stream: stream })
    }
}

fn make_callback<T>(
    sample_rate: u32,
    channels: usize,
    controls: AudioControls,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo) + Send + 'static
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_index = 0_u64;
    let mut effect_remaining = 0_u32;
    let mut effect_phase = 0.0_f32;
    let mut synth_frequency = 440.0_f32;
    move |output, _| {
        let enabled = controls.music_enabled.load(Ordering::Relaxed);
        let paused = controls.paused.load(Ordering::Relaxed);
        let volume = safe_volume(f32::from_bits(
            controls.music_volume_bits.load(Ordering::Relaxed),
        ));
        let effects_enabled = controls.effects_enabled.load(Ordering::Relaxed);
        let effects_volume = safe_volume(f32::from_bits(
            controls.effects_volume_bits.load(Ordering::Relaxed),
        ));
        for frame in output.chunks_mut(channels.max(1)) {
            let trigger = controls.effect_trigger.swap(0, Ordering::Relaxed);
            if trigger != 0 && effects_enabled {
                synth_frequency = effect_frequency(trigger);
                effect_phase = 0.0;
                effect_remaining = (sample_rate / 12).max(1);
            }
            let music = if enabled && !paused {
                bytebeat::sample_at(sample_index, sample_rate).clamp(-1.0, 1.0) * volume
            } else {
                0.0
            };
            if enabled && !paused {
                sample_index = sample_index.wrapping_add(1);
            }
            let effect = if effect_remaining > 0 && effects_enabled {
                let progress = 1.0 - effect_remaining as f32 / (sample_rate / 12).max(1) as f32;
                let envelope = (1.0 - progress).max(0.0);
                let value = effect_phase.sin() * envelope * effects_volume * 0.16;
                effect_phase += std::f32::consts::TAU * synth_frequency / sample_rate as f32;
                effect_remaining -= 1;
                value
            } else {
                if !effects_enabled {
                    effect_remaining = 0;
                }
                0.0
            };
            let sample = (music + effect).clamp(-1.0, 1.0);
            let converted = T::from_sample(sample);
            frame.fill(converted);
        }
    }
}

fn effect_frequency(id: u32) -> f32 {
    match id {
        1 => 520.0,
        2 => 720.0,
        3 => 190.0,
        4 => 260.0,
        5 => 620.0,
        6 => 760.0,
        7 => 880.0,
        8 => 980.0,
        9 => 110.0,
        10 => 440.0,
        _ => 330.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_audio_controls_clamp_volume() {
        let controls = AudioControls::default();
        controls.set_volume(1.5);
        controls.set_effects_volume(-0.2);
        assert_eq!(
            f32::from_bits(controls.music_volume_bits.load(Ordering::Relaxed)),
            1.0
        );
        assert_eq!(
            f32::from_bits(controls.effects_volume_bits.load(Ordering::Relaxed)),
            0.0
        );
    }
}
