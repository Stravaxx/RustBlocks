use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PieceKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceKind {
    pub const ALL: [Self; 7] = [
        Self::I,
        Self::O,
        Self::T,
        Self::S,
        Self::Z,
        Self::J,
        Self::L,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::I => "I",
            Self::O => "O",
            Self::T => "T",
            Self::S => "S",
            Self::Z => "Z",
            Self::J => "J",
            Self::L => "L",
        }
    }

    pub fn color(self) -> (u8, u8, u8) {
        match self {
            Self::I => (72, 205, 230),
            Self::O => (246, 202, 71),
            Self::T => (173, 105, 218),
            Self::S => (91, 201, 126),
            Self::Z => (235, 91, 95),
            Self::J => (82, 124, 224),
            Self::L => (238, 151, 77),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Rotation {
    North,
    East,
    South,
    West,
}

impl Rotation {
    pub fn clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
    pub fn counter_clockwise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Piece {
    pub kind: PieceKind,
    pub rotation: Rotation,
    pub x: i32,
    pub y: i32,
}

impl Piece {
    pub fn spawn(kind: PieceKind) -> Self {
        Self {
            kind,
            rotation: Rotation::North,
            x: 3,
            y: 1,
        }
    }

    pub fn cells(self) -> [(i32, i32); 4] {
        let base = match self.kind {
            PieceKind::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            PieceKind::O => [(1, 0), (2, 0), (1, 1), (2, 1)],
            PieceKind::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            PieceKind::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            PieceKind::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
            PieceKind::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            PieceKind::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        };
        if self.kind == PieceKind::O {
            return base.map(|(x, y)| (x + self.x, y + self.y));
        }
        let rotated = base.map(|(x, y)| match self.rotation {
            Rotation::North => (x, y),
            Rotation::East => (2 - y, x),
            Rotation::South => (2 - x, 2 - y),
            Rotation::West => (y, 2 - x),
        });
        rotated.map(|(x, y)| (x + self.x, y + self.y))
    }
}

pub(super) fn kick_tests(kind: PieceKind, from: Rotation, to: Rotation) -> &'static [(i32, i32)] {
    const J0R: &[(i32, i32)] = &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)];
    const JR0: &[(i32, i32)] = &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)];
    const JR2: &[(i32, i32)] = &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)];
    const J2R: &[(i32, i32)] = &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)];
    const J2L: &[(i32, i32)] = &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)];
    const JL2: &[(i32, i32)] = &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)];
    const JL0: &[(i32, i32)] = &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)];
    const J0L: &[(i32, i32)] = &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)];
    const I0R: &[(i32, i32)] = &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)];
    const IR0: &[(i32, i32)] = &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)];
    const IR2: &[(i32, i32)] = &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)];
    const I2R: &[(i32, i32)] = &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)];
    const I2L: &[(i32, i32)] = &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)];
    const IL2: &[(i32, i32)] = &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)];
    const IL0: &[(i32, i32)] = &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)];
    const I0L: &[(i32, i32)] = &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)];

    use Rotation::*;
    if kind == PieceKind::O {
        return &[(0, 0)];
    }
    match (kind == PieceKind::I, from, to) {
        (true, North, East) => I0R,
        (true, East, North) => IR0,
        (true, East, South) => IR2,
        (true, South, East) => I2R,
        (true, South, West) => I2L,
        (true, West, South) => IL2,
        (true, West, North) => IL0,
        (true, North, West) => I0L,
        (false, North, East) => J0R,
        (false, East, North) => JR0,
        (false, East, South) => JR2,
        (false, South, East) => J2R,
        (false, South, West) => J2L,
        (false, West, South) => JL2,
        (false, West, North) => JL0,
        (false, North, West) => J0L,
        _ => &[(0, 0)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_piece_has_four_distinct_cells() {
        for kind in PieceKind::ALL {
            let cells = Piece::spawn(kind).cells();
            for i in 0..4 {
                for j in i + 1..4 {
                    assert_ne!(cells[i], cells[j], "{kind:?}");
                }
            }
        }
    }

    #[test]
    fn rotation_cycles_back_to_north() {
        let mut rotation = Rotation::North;
        for _ in 0..4 {
            rotation = rotation.clockwise();
        }
        assert_eq!(rotation, Rotation::North);
    }
}
