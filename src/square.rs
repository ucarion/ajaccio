use std::ops::{Add, Sub};

use bitboard::Bitboard;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Square(u8);

impl Square {
    pub fn new(square_index: u8) -> Square {
        Square(square_index)
    }

    /// Makes a Square from a (file, rank) pair. To represent "a8", pass (0, 7).
    pub fn from_coords(file: u8, rank: u8) -> Square {
        Square(file + rank * 8)
    }

    /// Makes a Square from Standard Algebraic Notation (e.g. "a8").
    pub fn from_san(san: &str) -> Square {
        let san: Vec<_> = san.chars().collect();
        let file = match san[0] {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!("Unknown file: {:?}", san[0])
        };

        let rank = match san[1] {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => panic!("Unknown rank: {:?}", san[1])
        };

        Square::from_coords(file, rank)
    }

    pub fn to_bitboard(self) -> Bitboard {
        Bitboard::new(1 << self.0)
    }

    pub fn rank(self) -> u8 {
        self.0 / 8
    }

    pub fn file(self) -> u8 {
        self.0 % 8
    }
}

impl Add<u8> for Square {
    type Output = Square;

    fn add(self, rhs: u8) -> Square {
        Square(self.0 + rhs)
    }
}

impl Sub<u8> for Square {
    type Output = Square;

    fn sub(self, rhs: u8) -> Square {
        Square(self.0 - rhs)
    }
}

#[test]
fn san_square_parsing() {
    assert_eq!(Square::new(4 + 2 * 8), Square::from_san("e3"));
    assert_eq!(Square::new(4 + 2 * 8), Square::from_coords(4, 2));
}

