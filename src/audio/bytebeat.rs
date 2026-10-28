pub const DEFAULT_SAMPLE_RATE: u32 = 32_000;
const NOTES: [f64; 64] = [
    13., 13., 8., 9., 11., 11., 9., 8., 6., 6., 6., 9., 13., 13., 11., 9., 8., 8., 8., 9., 11.,
    11., 13., 13., 9., 9., 6., -1., 6., 6., -1., -1., -1., 11., 11., 14., 18., 18., 16., 14., 13.,
    13., 13., 9., 13., -1., 11., 9., 8., 8., 8., 9., 11., 11., 13., 13., 9., 9., 6., -1., 6., 6.,
    -1., -1.,
];
const BASS: [f64; 16] = [
    13., 13., 18., 18., 17., 13., 18., -1., 11., 11., 9., 9., 8., 1., 6., 6.,
];

fn js_to_i32(value: f64) -> i32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    let modulo = value.trunc().rem_euclid(4_294_967_296.0) as u32;
    modulo as i32
}

fn shifted_index(t: f64, divisor: f64, shift: u32, mask: i32) -> usize {
    ((js_to_i32(t / divisor) >> shift) & mask) as usize
}

/// Evaluates the Dollchan expression at the zero-based output sample counter.
/// The expression's JS `>>` operands use ToInt32 semantics; musical time is
/// therefore the sample count, with the original `t / 1.2` scaling preserved.
pub fn sample_at(t: u64, sample_rate: u32) -> f32 {
    if sample_rate == 0 {
        return 0.0;
    }
    let t = t as f64 * (f64::from(DEFAULT_SAMPLE_RATE) / f64::from(sample_rate));
    let m = -t;
    let u = 0.83_f64;
    let melody_step = shifted_index(t, 1.2, 13, 63);
    let k = [18.0, 18.0, 20.0, 9.0][shifted_index(t, 1.2, 13, 3)];
    let melody_note = if NOTES[melody_step] == -1.0 {
        m
    } else {
        NOTES[melody_step]
    };
    let v = u * 2.0 * t * 2.0_f64.powf(melody_note / 12.0) / 128.0 * std::f64::consts::PI;
    let bass_step = shifted_index(t, 1.2, 15, 15);
    let alternating = 1.0 + f64::from(((js_to_i32(t / 1.2) >> 13) & 1) as u8);
    let octave = if (js_to_i32(t / 1.2) >> 15) & 15 >= 8 {
        2.0
    } else {
        1.0
    };
    let bass_note = if BASS[bass_step] == -1.0 {
        k
    } else {
        BASS[bass_step]
    };
    let v2 = u * alternating * octave * t * 2.0_f64.powf(bass_note / 12.0) / 512.0
        * std::f64::consts::PI;
    let output = (v / 2.0 + (v * 1.5).sin() - (v * 2.0).cos().atan()).sin() / 5.0
        + (v2 + (v2 / 2.0).sin() * (32.0 - t / 1.2 / 256.0 % 32.0) / 6.0).sin() / 2.0;
    if output.is_finite() {
        output.clamp(-1.0, 1.0) as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_is_deterministic_finite_and_bounded() {
        for t in [
            0,
            1,
            100,
            13_999,
            44_100,
            1_000_000,
            u32::MAX as u64,
            u64::MAX,
        ] {
            let a = sample_at(t, DEFAULT_SAMPLE_RATE);
            assert_eq!(a, sample_at(t, DEFAULT_SAMPLE_RATE));
            assert!(
                a.is_finite() && (-1.0..=1.0).contains(&a),
                "t={t}, sample={a}"
            );
        }
    }

    #[test]
    fn zero_sample_rate_is_safe() {
        assert_eq!(sample_at(100, 0), 0.0);
    }

    #[test]
    fn samples_are_continuous_across_output_buffer_boundaries() {
        for boundary in [256, 1_024, 4_096, 16_384] {
            let before = sample_at(boundary - 1, DEFAULT_SAMPLE_RATE);
            let after = sample_at(boundary, DEFAULT_SAMPLE_RATE);
            assert!(before.is_finite() && after.is_finite());
            assert_eq!(after, sample_at(boundary, DEFAULT_SAMPLE_RATE));
        }
    }
}
