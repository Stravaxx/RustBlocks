# Gameplay

The board is ten columns wide with 20 visible rows and four spawn rows above
them. The I, O, T, S, Z, J and L pieces are generated from a deterministic
seven-bag sequence. The hold slot can be used once until a piece locks. Four
future pieces are displayed.

Rotations use the standard SRS transition kick offsets for I and JLSTZ pieces,
expressed in the engine's downward-positive coordinate system. O rotation is
visually unchanged. The piece geometry uses compact local coordinates; T-spin
recognition and spin bonuses are not implemented.

## Controls and timing

Keyboard controls are listed in the README and help screen. Gravity is elapsed
time based, independent of display refresh. Left/right repeat and soft drop
are polled from the frame loop; exact DAS/ARR customization is not yet exposed.
Lock delay is 0.5 seconds, with a 15 reset cap. Pause freezes simulation and
the soundtrack.

## Modes and score

- **Classic:** endless progression until top-out.
- **Marathon:** ends at 150 cleared lines or top-out.
- **Sprint 40:** ends after 40 cleared lines or top-out.
- **Ultra:** 120 seconds of score attack.
- **Zen:** slow fixed gravity, still ends on top-out.
- **Custom:** currently uses standard gravity and scoring; editable rule
  parameters are not yet exposed.

Single/double/triple/four-line clears award 100/300/500/800 points multiplied
by `level + 1`. Each ten lines raises the level. Consecutive non-empty clears
award 50 times combo count times `level + 1`; a no-clear lock resets the combo.
Soft drop gives one point per cell and hard drop gives two. Arithmetic
saturates at `u64::MAX`. T-spin scoring, back-to-back bonuses, detailed mode
records and an animated clear phase are known omissions.
