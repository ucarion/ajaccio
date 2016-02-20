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

fn is_in_bounds(file: i8, rank: i8) -> bool {
    1 <= file && file <= 6 && 1 <= rank && rank <= 6
}

pub fn bishop_attacks(square: Square) -> Bitboard {
    let start = (square.file() as i8, square.rank() as i8);

    diagonal_attacks(start, 1, 1) |
        diagonal_attacks(start, 1, -1) |
        diagonal_attacks(start, -1, 1) |
        diagonal_attacks(start, -1, -1)
}

fn diagonal_attacks(start: (i8, i8), dx: i8, dy: i8) -> Bitboard {
    let mut result = Bitboard::new(0);
    let mut cursor = start;
    loop {
        cursor = (cursor.0 + dx, cursor.1 + dy);

        if is_in_bounds(cursor.0, cursor.1) {
            result = result | Square::from_coords(cursor.0 as u8, cursor.1 as u8).to_bitboard();
        } else {
            break;
        }
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

#[test]
fn test_bishop_attacks() {
    let a1 = Bitboard::new(18049651735527936);
    let e4 = Bitboard::new(637888545440768);
    let h8 = Bitboard::new(18049651735527936);

    assert_eq!(a1, bishop_attacks(Square::from_san("a1")));
    assert_eq!(e4, bishop_attacks(Square::from_san("e4")));
    assert_eq!(h8, bishop_attacks(Square::from_san("h8")));
}
