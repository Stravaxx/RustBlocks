mod board;
mod piece;
mod rng;
mod scoring;

pub use board::{BOARD_HEIGHT, BOARD_WIDTH, VISIBLE_HEIGHT};
pub use piece::{Piece, PieceKind, Rotation};
pub use rng::SevenBag;
pub use scoring::{GameMode, GameStats, ScoreEvent};

use board::Board;
use serde::{Deserialize, Serialize};

const LOCK_DELAY: f32 = 0.5;
const MAX_LOCK_RESETS: u8 = 15;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub board: Board,
    pub active: Piece,
    pub queue: Vec<PieceKind>,
    pub hold: Option<PieceKind>,
    pub hold_used: bool,
    pub stats: GameStats,
    pub mode: GameMode,
    pub paused: bool,
    pub over: bool,
    pub elapsed: f64,
    rng: SevenBag,
    gravity_timer: f32,
    lock_timer: f32,
    lock_resets: u8,
}

impl Game {
    pub fn new(mode: GameMode, seed: u64) -> Self {
        let mut rng = SevenBag::new(seed);
        let mut queue = Vec::with_capacity(6);
        for _ in 0..7 {
            queue.push(rng.next_piece());
        }
        let active = Piece::spawn(queue.remove(0));
        Self {
            board: Board::default(),
            active,
            queue,
            hold: None,
            hold_used: false,
            stats: GameStats::default(),
            mode,
            paused: false,
            over: false,
            elapsed: 0.0,
            rng,
            gravity_timer: 0.0,
            lock_timer: 0.0,
            lock_resets: 0,
        }
    }

    pub fn tick(&mut self, dt: f32) -> Vec<ScoreEvent> {
        if self.paused || self.over || !dt.is_finite() || dt <= 0.0 {
            return Vec::new();
        }
        let dt = dt.min(0.1);
        self.elapsed += f64::from(dt);
        if self.mode == GameMode::Ultra && self.elapsed >= 120.0 {
            self.over = true;
            return Vec::new();
        }
        self.gravity_timer += dt;
        if self.gravity_timer >= self.gravity_interval() {
            self.gravity_timer = 0.0;
            if !self.try_move(0, 1) {
                self.lock_timer += dt;
                if self.lock_timer >= LOCK_DELAY {
                    return self.lock_piece();
                }
            } else {
                self.lock_timer = 0.0;
            }
        } else if self.is_grounded() {
            self.lock_timer += dt;
            if self.lock_timer >= LOCK_DELAY {
                return self.lock_piece();
            }
        }
        Vec::new()
    }

    pub fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let mut candidate = self.active;
        candidate.x += dx;
        candidate.y += dy;
        if self.board.can_place(candidate) {
            self.active = candidate;
            self.reset_lock_if_grounded();
            true
        } else {
            false
        }
    }

    pub fn soft_drop(&mut self) -> Vec<ScoreEvent> {
        if self.try_move(0, 1) {
            self.stats.score = self.stats.score.saturating_add(1);
            self.gravity_timer = 0.0;
            Vec::new()
        } else {
            self.lock_timer = LOCK_DELAY;
            self.lock_piece()
        }
    }

    pub fn hard_drop(&mut self) -> Vec<ScoreEvent> {
        if self.paused || self.over {
            return Vec::new();
        }
        let mut cells = 0_u64;
        while self.try_move(0, 1) {
            cells += 1;
        }
        self.stats.score = self.stats.score.saturating_add(cells.saturating_mul(2));
        self.lock_piece()
    }

    pub fn rotate(&mut self, clockwise: bool) -> bool {
        if self.paused || self.over {
            return false;
        }
        let old_rotation = self.active.rotation;
        let next = if clockwise {
            old_rotation.clockwise()
        } else {
            old_rotation.counter_clockwise()
        };
        let kicks = piece::kick_tests(self.active.kind, old_rotation, next);
        for (dx, dy) in kicks {
            let mut candidate = self.active;
            candidate.rotation = next;
            candidate.x += dx;
            candidate.y += dy;
            if self.board.can_place(candidate) {
                self.active = candidate;
                self.reset_lock_if_grounded();
                return true;
            }
        }
        false
    }

    pub fn hold_piece(&mut self) -> bool {
        if self.hold_used || self.paused || self.over {
            return false;
        }
        let current = self.active.kind;
        if let Some(held) = self.hold {
            self.active = Piece::spawn(held);
            self.hold = Some(current);
        } else {
            self.hold = Some(current);
            self.spawn_next();
        }
        self.hold_used = true;
        if !self.board.can_place(self.active) {
            self.over = true;
        }
        true
    }

    pub fn toggle_pause(&mut self) {
        if !self.over {
            self.paused = !self.paused;
        }
    }

    fn gravity_interval(&self) -> f32 {
        if self.mode == GameMode::Zen {
            return 0.8;
        }
        (0.8_f32 * 0.82_f32.powi(self.stats.level.min(100) as i32)).max(0.045)
    }

    fn is_grounded(&self) -> bool {
        let mut below = self.active;
        below.y += 1;
        !self.board.can_place(below)
    }

    fn reset_lock_if_grounded(&mut self) {
        if self.is_grounded() && self.lock_resets < MAX_LOCK_RESETS {
            self.lock_timer = 0.0;
            self.lock_resets += 1;
        }
    }

    fn lock_piece(&mut self) -> Vec<ScoreEvent> {
        self.board.lock(self.active);
        let cleared = self.board.clear_full_lines();
        let events = self.stats.record_clear(cleared);
        if self.mode == GameMode::Sprint && self.stats.lines >= 40 {
            self.over = true;
        }
        if self.mode == GameMode::Marathon && self.stats.lines >= 150 {
            self.over = true;
        }
        self.spawn_next();
        events
    }

    fn spawn_next(&mut self) {
        self.active = Piece::spawn(self.queue.remove(0));
        self.queue.push(self.rng.next_piece());
        self.hold_used = false;
        self.gravity_timer = 0.0;
        self.lock_timer = 0.0;
        self.lock_resets = 0;
        if !self.board.can_place(self.active) {
            self.over = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_a_valid_piece_and_preview_queue() {
        let game = Game::new(GameMode::Classic, 7);
        assert_eq!(game.queue.len(), 6);
        assert!(game.board.can_place(game.active));
    }

    #[test]
    fn hard_drop_locks_and_scores() {
        let mut game = Game::new(GameMode::Classic, 19);
        let events = game.hard_drop();
        assert!(events.is_empty());
        assert!(game.stats.score >= 2);
        assert!(game.board.occupied_count() > 0);
    }

    #[test]
    fn hold_cannot_be_reused_until_a_lock() {
        let mut game = Game::new(GameMode::Classic, 3);
        assert!(game.hold_piece());
        assert!(!game.hold_piece());
        game.hard_drop();
        assert!(game.hold_piece());
    }

    #[test]
    fn pause_prevents_simulation_time_from_advancing() {
        let mut game = Game::new(GameMode::Classic, 9);
        game.toggle_pause();
        game.tick(1.0);
        assert_eq!(game.elapsed, 0.0);
    }

    #[test]
    fn same_seed_produces_same_sequence() {
        let first = Game::new(GameMode::Classic, 42);
        let second = Game::new(GameMode::Classic, 42);
        assert_eq!(first.active, second.active);
        assert_eq!(first.queue, second.queue);
    }

    #[test]
    fn rotation_uses_the_i_piece_wall_kick_at_the_left_edge() {
        let mut game = Game::new(GameMode::Classic, 10);
        game.active = Piece {
            kind: PieceKind::I,
            rotation: Rotation::East,
            x: -1,
            y: 5,
        };
        assert!(game.rotate(true));
        assert_eq!(game.active.rotation, Rotation::South);
        assert_eq!(game.active.x, 1);
        assert!(game.board.can_place(game.active));
    }

    #[test]
    fn rotation_uses_a_floor_kick_when_blocked_at_bottom() {
        let mut game = Game::new(GameMode::Classic, 11);
        game.active = Piece {
            kind: PieceKind::T,
            rotation: Rotation::North,
            x: 3,
            y: 22,
        };
        assert!(game.rotate(true));
        assert_eq!(game.active.y, 21);
        assert!(game.board.can_place(game.active));
    }

    #[test]
    fn identical_inputs_and_seeds_produce_identical_simulations() {
        let mut first = Game::new(GameMode::Classic, 884);
        let mut second = Game::new(GameMode::Classic, 884);
        for _ in 0..8 {
            first.hard_drop();
            second.hard_drop();
            assert_eq!(first.board, second.board);
            assert_eq!(first.active, second.active);
            assert_eq!(first.queue, second.queue);
            assert_eq!(first.stats.score, second.stats.score);
        }
    }

    #[test]
    fn game_ends_when_the_next_piece_cannot_spawn() {
        let mut game = Game::new(GameMode::Classic, 31);
        game.board.lock(Piece {
            kind: PieceKind::O,
            rotation: Rotation::North,
            x: 3,
            y: 1,
        });
        game.active = Piece {
            kind: PieceKind::I,
            rotation: Rotation::North,
            x: 3,
            y: 18,
        };
        game.hard_drop();
        assert!(game.over);
    }
}
