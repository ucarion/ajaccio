use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::fmt;

use square::Square;

// The internal u64 is `pub` for pragmatic reasons, but let's avoid using it too much.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bitboard(pub u64);

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

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Bitboard {
        Bitboard(!self.0)
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

    pub fn is_occupied(&self, square: Square) -> bool {
        (self.clone() & square.to_bitboard()).is_nonempty()
    }

    pub fn num_occupied_squares(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn squares(self) -> SquaresIter {
        SquaresIter { bitboard: self }
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let line = "+---+---+---+---+---+---+---+---+\n";
        try!(write!(f, "{}", line));

        for rank in (0..8).rev() {
            try!(write!(f, "|"));

            for file in 0..8 {
                let sq = Square::from_coords(file, rank);
                let to_write = if self.is_occupied(sq) {
                    'X'
                } else {
                    ' '
                };

                try!(write!(f, " {} |", to_write));
            }

            try!(write!(f, "\n"));
            try!(write!(f, "{}", line));
        }

        Ok(())
    }
}

pub struct SquaresIter {
    bitboard: Bitboard
}

impl Iterator for SquaresIter {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.bitboard.is_empty() {
            None
        } else {
            let top_one_square = Square::new(63 - self.bitboard.0.leading_zeros() as u8);
            let without_top = self.bitboard ^ top_one_square.to_bitboard();

            self.bitboard = without_top;
            Some(top_one_square)
        }
    }
}

impl SquaresIter {
    fn new(bitboard: Bitboard) -> SquaresIter {
        SquaresIter { bitboard: bitboard }
    }
}

#[test]
fn test_squares_iter() {
    let a = Square::from_san("e6");
    let b = Square::from_san("c2");

    let expected = vec![a, b];
    let actual = SquaresIter::new(a.to_bitboard() | b.to_bitboard()).collect::<Vec<_>>();
    assert_eq!(expected, actual);
}
