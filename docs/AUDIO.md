# Audio and bytebeat

The generated music is evaluated by `audio::bytebeat::sample_at` and the live
CPAL callback; the diagnostic binary uses the same evaluator. No prerecorded
track is substituted.

## Porting convention

The supplied Dollchan expression is evaluated using binary64 intermediates
(Rust `f64`), then converted to `f32` for the callback. `t` is the monotonically
increasing zero-based output sample counter. The original expression divides
that counter by `1.2` before its bit shifts. JavaScript bitwise shifts coerce
their left operand with ToInt32 and mask the shift count; the implementation
uses truncation modulo 2^32 and signed arithmetic right shift. Array indexes
mask with `& 63`, `& 3` and `& 15`; the exact arrays are held in the source.
In the note tables, the sentinel representing the expression's `m` variable is
substituted with `m = -t`; the bass table sentinel maps to `k`.

The `sample_rate` argument adapts the sample-counter timebase to the 44,100 Hz
reference (`t * 44100 / actual_rate`), so the note progression keeps its
intended duration on a device with a different output rate. At 44.1 kHz the
expression is evaluated without time scaling. This convention is a practical
desktop interpretation; no authoritative Dollchan runtime sample-rate
contract was supplied. At rates other than 44.1 kHz, the counter is fractional
before ToInt32 conversion and JS truncation is preserved.

The mathematical output has a bounded sum of two sine terms; output is
guarded for non-finite values and clamped to [-1, 1] before device conversion.
Volume is a separate linear gain. There is no extra filtering or pitch
processing. Pausing emits silence while preserving the sample counter; resume
continues at the next sample. The callback performs no I/O and no allocation.
Audio errors are logged and disable further music output for that stream.

The diagnostic WAV is mono PCM16 little-endian. Example:

```sh
cargo run --release --bin rustblocks-audio -- preview.wav 10 44100
```

The application also synthesizes short, original sine-envelope effects for
movement, rotation, drops, line clears, combos, level changes, game over and
menu actions, with an independent effects volume/enable setting. The supplied
expression's copyright/licensing provenance was not included with the
requirements.
