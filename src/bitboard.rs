use std::ops::{BitAnd, BitOr, BitXor};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bitboard(u64);

impl BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl Bitboard {
    pub fn new(bitmask: u64) -> Bitboard {
        Bitboard(bitmask)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_nonempty(&self) -> bool {
        !self.is_empty()
    }
}
