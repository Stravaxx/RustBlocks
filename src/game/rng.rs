use super::PieceKind;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SevenBag {
    state: u64,
    bag: [PieceKind; 7],
    index: usize,
}

impl SevenBag {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.max(1),
            bag: PieceKind::ALL,
            index: 7,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_piece(&mut self) -> PieceKind {
        if self.index == 7 {
            for i in (1..7).rev() {
                let j = (self.next_u64() as usize) % (i + 1);
                self.bag.swap(i, j);
            }
            self.index = 0;
        }
        let result = self.bag[self.index];
        self.index += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_bag_contains_every_piece_once() {
        let mut bag = SevenBag::new(123);
        let values: Vec<_> = (0..7).map(|_| bag.next_piece()).collect();
        for kind in PieceKind::ALL {
            assert_eq!(values.iter().filter(|&&v| v == kind).count(), 1);
        }
    }
}
