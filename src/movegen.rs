use bitboard::Bitboard;
use square::Square;

fn white_pawn_attacks(square: Square) -> Bitboard {
    let rank = square.rank() as i8;
    let file = square.file() as i8;

    let mut result = Bitboard::new(0);

    result = add_if_in_bounds(result, file + 1, rank + 1);
    result = add_if_in_bounds(result, file - 1, rank + 1);

    result
}

fn black_pawn_attacks(square: Square) -> Bitboard {
    let rank = square.rank() as i8;
    let file = square.file() as i8;

    let mut result = Bitboard::new(0);

    result = add_if_in_bounds(result, file + 1, rank - 1);
    result = add_if_in_bounds(result, file - 1, rank - 1);

    result
}

fn white_pawn_moves(square: Square) -> Bitboard {
    if square.rank() == 1 {
        (square + 8).to_bitboard() | (square + 16).to_bitboard()
    } else {
        (square + 8).to_bitboard()
    }
}

fn black_pawn_moves(square: Square) -> Bitboard {
    if square.rank() == 6 {
        (square - 8).to_bitboard() | (square - 16).to_bitboard()
    } else {
        (square - 8).to_bitboard()
    }
}

fn knight_moves(square: Square) -> Bitboard {
    let mut result = Bitboard::new(0);
    let file = square.file() as i8;
    let rank = square.rank() as i8;

    result = add_if_in_bounds(result, file + 1, rank + 2);
    result = add_if_in_bounds(result, file + 1, rank - 2);
    result = add_if_in_bounds(result, file - 1, rank + 2);
    result = add_if_in_bounds(result, file - 1, rank - 2);
    result = add_if_in_bounds(result, file + 2, rank + 1);
    result = add_if_in_bounds(result, file + 2, rank - 1);
    result = add_if_in_bounds(result, file - 2, rank + 1);
    result = add_if_in_bounds(result, file - 2, rank - 1);

    result
}

fn king_moves(square: Square) -> Bitboard {
    let mut result = Bitboard::new(0);
    let file = square.file() as i8;
    let rank = square.rank() as i8;

    result = add_if_in_bounds(result, file + 1, rank + 1);
    result = add_if_in_bounds(result, file + 1, rank - 1);
    result = add_if_in_bounds(result, file - 1, rank + 1);
    result = add_if_in_bounds(result, file - 1, rank - 1);
    result = add_if_in_bounds(result, file + 1, rank);
    result = add_if_in_bounds(result, file - 1, rank);
    result = add_if_in_bounds(result, file, rank + 1);
    result = add_if_in_bounds(result, file, rank - 1);

    result
}

fn coords_in_bounds(file: i8, rank: i8) -> bool {
    0 <= file && file < 8 && 0 <= rank && rank < 8
}

fn add_if_in_bounds(bitboard: Bitboard, file: i8, rank: i8) -> Bitboard {
    if coords_in_bounds(file, rank) {
        bitboard | Square::from_coords(file as u8, rank as u8).to_bitboard()
    } else {
        bitboard
    }
}

#[test]
fn test_pawn_attacks() {
    let e4 = Bitboard::new(171798691840);
    let a2 = Bitboard::new(131072);
    let h7 = Bitboard::new(4611686018427387904);

    assert_eq!(e4, white_pawn_attacks(Square::from_san("e4")));
    assert_eq!(a2, white_pawn_attacks(Square::from_san("a2")));
    assert_eq!(h7, white_pawn_attacks(Square::from_san("h7")));

    let e4 = Bitboard::new(2621440);
    let a2 = Bitboard::new(2);
    let h7 = Bitboard::new(70368744177664);

    assert_eq!(e4, black_pawn_attacks(Square::from_san("e4")));
    assert_eq!(a2, black_pawn_attacks(Square::from_san("a2")));
    assert_eq!(h7, black_pawn_attacks(Square::from_san("h7")));
}

#[test]
fn test_pawn_moves() {
    let expected = Square::from_san("e3").to_bitboard()
            | Square::from_san("e4").to_bitboard();
    assert_eq!(expected, white_pawn_moves(Square::from_san("e2")));

    let expected = Square::from_san("e4").to_bitboard();
    assert_eq!(expected, white_pawn_moves(Square::from_san("e3")));

    let expected = Square::from_san("e6").to_bitboard()
            | Square::from_san("e5").to_bitboard();
    assert_eq!(expected, black_pawn_moves(Square::from_san("e7")));

    let expected = Square::from_san("e5").to_bitboard();
    assert_eq!(expected, black_pawn_moves(Square::from_san("e6")));
}

#[test]
fn test_knight_moves() {
    let e4 = Bitboard::new(44272527353856);
    let a1 = Bitboard::new(132096);
    let e7 = Bitboard::new(4899991333168480256);
    assert_eq!(e4, knight_moves(Square::from_san("e4")));
    assert_eq!(a1, knight_moves(Square::from_san("a1")));
    assert_eq!(e7, knight_moves(Square::from_san("e7")));
}

#[test]
fn test_king_moves() {
    let e4 = Bitboard::new(241192927232);
    let a1 = Bitboard::new(770);

    assert_eq!(e4, king_moves(Square::from_san("e4")));
    assert_eq!(a1, king_moves(Square::from_san("a1")));
}
