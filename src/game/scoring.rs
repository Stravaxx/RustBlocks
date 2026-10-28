use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Marathon,
    Sprint,
    Ultra,
    Zen,
    Custom,
}

impl GameMode {
    pub const ALL: [Self; 6] = [
        Self::Classic,
        Self::Marathon,
        Self::Sprint,
        Self::Ultra,
        Self::Zen,
        Self::Custom,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Marathon => "Marathon",
            Self::Sprint => "Sprint 40",
            Self::Ultra => "Ultra 2 min",
            Self::Zen => "Zen",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScoreEvent {
    Single,
    Double,
    Triple,
    Tetris,
    Combo(u32),
    LevelUp(u32),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameStats {
    pub score: u64,
    pub lines: u32,
    pub level: u32,
    pub combo: i32,
    pub pieces: u64,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            score: 0,
            lines: 0,
            level: 0,
            combo: -1,
            pieces: 0,
        }
    }
}

impl GameStats {
    pub fn record_clear(&mut self, lines: u8) -> Vec<ScoreEvent> {
        self.pieces = self.pieces.saturating_add(1);
        if lines == 0 {
            self.combo = -1;
            return Vec::new();
        }
        let old_level = self.level;
        let base = match lines {
            1 => 100_u64,
            2 => 300,
            3 => 500,
            _ => 800,
        };
        self.score = self
            .score
            .saturating_add(base.saturating_mul(u64::from(self.level) + 1));
        self.lines = self.lines.saturating_add(u32::from(lines));
        self.level = self.lines / 10;
        self.combo += 1;
        let mut events = vec![match lines {
            1 => ScoreEvent::Single,
            2 => ScoreEvent::Double,
            3 => ScoreEvent::Triple,
            _ => ScoreEvent::Tetris,
        }];
        if self.combo > 0 {
            self.score = self.score.saturating_add(
                50_u64
                    .saturating_mul(self.combo as u64)
                    .saturating_mul(u64::from(self.level + 1)),
            );
            events.push(ScoreEvent::Combo(self.combo as u32));
        }
        if self.level > old_level {
            events.push(ScoreEvent::LevelUp(self.level));
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoring_updates_level_and_combo() {
        let mut stats = GameStats::default();
        assert!(stats.record_clear(1).contains(&ScoreEvent::Single));
        assert!(stats.record_clear(4).contains(&ScoreEvent::Combo(1)));
        assert_eq!(stats.lines, 5);
        assert!(stats.score >= 900);
    }

    #[test]
    fn score_saturates_instead_of_overflowing() {
        let mut stats = GameStats {
            score: u64::MAX,
            ..GameStats::default()
        };
        stats.record_clear(4);
        assert_eq!(stats.score, u64::MAX);
    }
}
