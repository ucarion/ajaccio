use bitboard::Bitboard;
use square::Square;

pub fn rook_attacks(square: Square) -> Bitboard {
    let mut result = Bitboard::new(0);

    for rank in (square.rank() + 1)..7 {
        result = result | Square::from_coords(square.file(), rank).to_bitboard();
    }

    for rank in 1..square.rank() {
        result = result | Square::from_coords(square.file(), rank).to_bitboard();
    }

    for file in (square.file() + 1)..7 {
        result = result | Square::from_coords(file, square.rank()).to_bitboard();
    }

    for file in 1..square.file() {
        result = result | Square::from_coords(file, square.rank()).to_bitboard();
    }

    result
}

#[test]
fn test_rook_attacks() {
    // TODO: Is there a better way to establish that I want these cases to work than to hard-code
    // the correct values?
    let a1 = Bitboard::new(282578800148862);
    let e4 = Bitboard::new(4521262379438080);
    let h8 = Bitboard::new(9115426935197958144);

    assert_eq!(a1, rook_attacks(Square::from_san("a1")));
    assert_eq!(e4, rook_attacks(Square::from_san("e4")));
    assert_eq!(h8, rook_attacks(Square::from_san("h8")));
}
