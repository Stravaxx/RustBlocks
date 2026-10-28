use super::Piece;
use serde::{Deserialize, Serialize};

pub const BOARD_WIDTH: usize = 10;
pub const VISIBLE_HEIGHT: usize = 20;
pub const HIDDEN_HEIGHT: usize = 4;
pub const BOARD_HEIGHT: usize = VISIBLE_HEIGHT + HIDDEN_HEIGHT;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Board {
    cells: [[Option<super::PieceKind>; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }
}

impl Board {
    pub fn cell(&self, x: usize, y: usize) -> Option<super::PieceKind> {
        self.cells
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .flatten()
    }

    pub fn can_place(&self, piece: Piece) -> bool {
        piece.cells().into_iter().all(|(x, y)| {
            x >= 0
                && x < BOARD_WIDTH as i32
                && y >= 0
                && y < BOARD_HEIGHT as i32
                && self.cells[y as usize][x as usize].is_none()
        })
    }

    pub fn lock(&mut self, piece: Piece) {
        for (x, y) in piece.cells() {
            if x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32 {
                self.cells[y as usize][x as usize] = Some(piece.kind);
            }
        }
    }

    pub fn clear_full_lines(&mut self) -> u8 {
        let mut kept = [[None; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut target = BOARD_HEIGHT;
        let mut cleared = 0;
        for source in (0..BOARD_HEIGHT).rev() {
            if self.cells[source].iter().all(Option::is_some) {
                cleared += 1;
            } else {
                target -= 1;
                kept[target] = self.cells[source];
            }
        }
        self.cells = kept;
        cleared
    }

    pub fn occupied_count(&self) -> usize {
        self.cells
            .iter()
            .flatten()
            .filter(|cell| cell.is_some())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{PieceKind, Rotation};

    #[test]
    fn rejects_wall_and_occupied_cell_collisions() {
        let mut board = Board::default();
        let piece = Piece {
            kind: PieceKind::O,
            rotation: Rotation::North,
            x: -2,
            y: 0,
        };
        assert!(!board.can_place(piece));
        board.lock(Piece {
            x: 4,
            y: 18,
            ..piece
        });
        assert!(!board.can_place(Piece {
            x: 4,
            y: 18,
            ..piece
        }));
    }

    #[test]
    fn clears_multiple_lines_and_compacts_rows() {
        let mut board = Board::default();
        for x in 0..BOARD_WIDTH {
            board.cells[BOARD_HEIGHT - 1][x] = Some(PieceKind::I);
            board.cells[BOARD_HEIGHT - 2][x] = Some(PieceKind::T);
        }
        board.cells[BOARD_HEIGHT - 3][0] = Some(PieceKind::J);
        assert_eq!(board.clear_full_lines(), 2);
        assert_eq!(board.cell(0, BOARD_HEIGHT - 1), Some(PieceKind::J));
        assert_eq!(board.occupied_count(), 1);
    }
}
