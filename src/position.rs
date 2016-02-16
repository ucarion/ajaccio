use fen;

#[derive(Debug, Default)]
struct Position {
    white: Army,
    black: Army,
    side_to_play: Color,
    white_can_oo: bool,
    white_can_ooo: bool,
    black_can_oo: bool,
    black_can_ooo: bool,
    en_passant: Option<Square>,
    halfmove_clock: u64,
    fullmove_number: u64
}

impl Position {
    fn from_fen(fen: &str) -> fen::FenResult<Position> {
        let mut position = Position::default();
        let fen_board = try!(fen::BoardState::from_fen(fen));

        for i in 0..64 {
            match fen_board.pieces[i] {
                None => {},
                Some(ref piece) => {
                    let army = match piece.color {
                        fen::Color::White => &mut position.white,
                        fen::Color::Black => &mut position.black
                    };

                    let bitboard = match piece.kind {
                        fen::PieceKind::Pawn => &mut army.pawns,
                        fen::PieceKind::Knight => &mut army.knights,
                        fen::PieceKind::Bishop => &mut army.bishops,
                        fen::PieceKind::Rook => &mut army.rooks,
                        fen::PieceKind::Queen => &mut army.queens,
                        fen::PieceKind::King => &mut army.king
                    };

                    bitboard.0 |= 1 << i;
                }
            }
        }

        position.side_to_play = match fen_board.side_to_play {
            fen::Color::White => Color::White,
            fen::Color::Black => Color::Black
        };

        position.white_can_oo = fen_board.white_can_oo;
        position.white_can_ooo = fen_board.white_can_ooo;
        position.black_can_oo = fen_board.black_can_oo;
        position.black_can_ooo = fen_board.black_can_ooo;
        position.en_passant = match fen_board.en_passant_square {
            None => None,
            Some(square) => Some(Square(square))
        };

        position.halfmove_clock = fen_board.halfmove_clock;
        position.fullmove_number = fen_board.fullmove_number;

        Ok(position)
    }
}

#[derive(Debug, Default)]
struct Army {
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    king: Bitboard
}

#[derive(Debug, Default)]
struct Bitboard(u64);

#[derive(Debug, Default, Eq, PartialEq)]
struct Square(u8);

impl Square {
    /// Makes a Square from a (file, rank) pair. To represent "a8", pass (0, 7).
    pub fn from_coords(file: u8, rank: u8) -> Square {
        Square(file + rank * 8)
    }

    /// Makes a Square from Standard Algebraic Notation (e.g. "a8").
    fn from_san(san: &str) -> Square {
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
}

#[derive(Debug, Eq, PartialEq)]
enum Color {
    White,
    Black
}

impl Default for Color {
    fn default() -> Color {
        Color::White
    }
}

#[test]
fn fen_parsing() {
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let position = Position::from_fen(fen).unwrap();

    assert_eq!(Color::Black, position.side_to_play);
    assert!(position.white_can_oo);
    assert!(position.white_can_ooo);
    assert!(position.black_can_oo);
    assert!(position.black_can_ooo);
    assert_eq!(Some(Square(20)), position.en_passant);
    assert_eq!(0, position.halfmove_clock);
    assert_eq!(1, position.fullmove_number);
}

#[test]
fn san_square_parsing() {
    assert_eq!(Square(4 + 2 * 8), Square::from_san("e3"));
    assert_eq!(Square(4 + 2 * 8), Square::from_coords(4, 2));
}
