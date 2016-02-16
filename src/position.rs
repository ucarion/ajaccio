use std::fmt;

use fen;
use bitboard::Bitboard;
use square::Square;

#[derive(Default)]
pub struct Position {
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
    pub fn from_fen(fen: &str) -> fen::FenResult<Position> {
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

                    let square_bitboard = Square::new(i as u8).to_bitboard();
                    *bitboard = bitboard.clone() | square_bitboard;
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
            Some(square) => Some(Square::new(square))
        };

        position.halfmove_clock = fen_board.halfmove_clock;
        position.fullmove_number = fen_board.fullmove_number;

        Ok(position)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let bitboard = square.to_bitboard();

        if (self.white.pawns & bitboard).is_nonempty() {
            Some(Piece::new(Color::White, PieceKind::Pawn))
        } else if (self.white.knights & bitboard).is_nonempty() {
            Some(Piece::new(Color::White, PieceKind::Knight))
        } else if (self.white.bishops& bitboard).is_nonempty() {
            Some(Piece::new(Color::White, PieceKind::Bishop))
        } else if (self.white.rooks & bitboard).is_nonempty() {
            Some(Piece::new(Color::White, PieceKind::Rook))
        } else if (self.white.queens & bitboard).is_nonempty() {
            Some(Piece::new(Color::White, PieceKind::Queen))
        } else if (self.white.king & bitboard).is_nonempty() {
            Some(Piece::new(Color::White, PieceKind::King))
        } else if (self.black.pawns & bitboard).is_nonempty() {
            Some(Piece::new(Color::Black, PieceKind::Pawn))
        } else if (self.black.knights & bitboard).is_nonempty() {
            Some(Piece::new(Color::Black, PieceKind::Knight))
        } else if (self.black.bishops& bitboard).is_nonempty() {
            Some(Piece::new(Color::Black, PieceKind::Bishop))
        } else if (self.black.rooks & bitboard).is_nonempty() {
            Some(Piece::new(Color::Black, PieceKind::Rook))
        } else if (self.black.queens & bitboard).is_nonempty() {
            Some(Piece::new(Color::Black, PieceKind::Queen))
        } else if (self.black.king & bitboard).is_nonempty() {
            Some(Piece::new(Color::Black, PieceKind::King))
        } else {
            None
        }
    }

    pub fn make_move(&mut self, motion: Move) {
        let from = self.piece_at(motion.from).unwrap();

        // update half-move counter -- this is done early so that the move being performed can
        // reset it to zero later
        match from.kind {
            PieceKind::Pawn => {
                self.halfmove_clock = 0;
            },

            _ => {
                self.halfmove_clock += 1;
            }
        }

        // update full-move number
        match from.color {
            Color::Black => {
                self.fullmove_number += 1;
            },

            _ => {}
        }

        // change the bitboard of any piece being captured
        match self.piece_at(motion.to) {
            Some(to) => {
                self.halfmove_clock = 0;

                let bitboard = self.get_bitboard(to);
                let bitmask = motion.to.to_bitboard();

                *bitboard = bitboard.clone() ^ bitmask;
            },

            None => {}
        };

        {
            // change the bitboard of the moving piece
            let bitboard = self.get_bitboard(from.clone());
            let bitmask = motion.from.to_bitboard() | motion.to.to_bitboard();

            *bitboard = bitboard.clone() ^ bitmask;
        }

        match from.kind {
            PieceKind::Pawn => {
                // handle en passant
                let (ep_file_start, ep_file_end) = match self.side_to_play {
                    Color::White => { (1, 3) },
                    Color::Black => { (6, 4) }
                };

                if motion.from.rank() == ep_file_start && motion.to.rank() == ep_file_end {
                    self.en_passant = Some(match self.side_to_play {
                        Color::White => motion.from + 8,
                        Color::Black => motion.from - 8
                    });
                }

                let promo_rank = match self.side_to_play {
                    Color::White => 7,
                    Color::Black => 0
                };

                if motion.to.rank() == promo_rank {
                    let army = match self.side_to_play {
                        Color::White => &mut self.white,
                        Color::Black => &mut self.black
                    };

                    let pawn_bitmask = motion.to.to_bitboard();
                    army.pawns = army.pawns ^ pawn_bitmask;

                    let promo_bitboard = army.get_bitboard_mut(motion.promote_to.unwrap());
                    let promo_bitmask = motion.to.to_bitboard();
                    *promo_bitboard = promo_bitboard.clone() | promo_bitmask;
                }
            },

            PieceKind::Rook => {
                // handle updating castling rights
                match self.side_to_play {
                    Color::White => {
                        if motion.from == Square::from_san("h1") {
                            self.white_can_oo = false;
                        }

                        if motion.from == Square::from_san("a1") {
                            self.white_can_ooo = false;
                        }
                    },

                    Color::Black => {
                        if motion.from == Square::from_san("h8") {
                            self.black_can_oo = false;
                        }

                        if motion.from == Square::from_san("a8") {
                            self.black_can_ooo = false;
                        }
                    }
                }

            }

            PieceKind::King => {
                // handle castling and update castling rights
                match self.side_to_play {
                    Color::White => {
                        let king_from = Square::from_san("e1");
                        let oo_to = Square::from_san("g1");
                        let ooo_to = Square::from_san("c1");

                        if motion.from == king_from && motion.to == oo_to {
                            let rook_bitmask = Square::from_san("h1").to_bitboard()
                                    | Square::from_san("f1").to_bitboard();
                            self.white.rooks = self.white.rooks ^ rook_bitmask;
                        }

                        if motion.from == king_from && motion.to == ooo_to {
                            let rook_bitmask = Square::from_san("a1").to_bitboard()
                                    | Square::from_san("d1").to_bitboard();
                            self.white.rooks = self.white.rooks ^ rook_bitmask;
                        }

                        self.white_can_oo = false;
                        self.white_can_ooo = false;
                    },

                    Color::Black => {
                        let king_from = Square::from_san("e8");
                        let oo_to = Square::from_san("g8");
                        let ooo_to = Square::from_san("c8");

                        if motion.from == king_from && motion.to == oo_to {
                            let rook_bitmask = Square::from_san("h8").to_bitboard()
                                    | Square::from_san("f8").to_bitboard();
                            self.white.rooks = self.white.rooks ^ rook_bitmask;
                        }

                        if motion.from == king_from && motion.to == ooo_to {
                            let rook_bitmask = Square::from_san("a8").to_bitboard()
                                    | Square::from_san("d8").to_bitboard();
                            self.white.rooks = self.white.rooks ^ rook_bitmask;
                        }

                        self.black_can_oo = false;
                        self.black_can_ooo = false;
                    }
                }
            }

            _ => {}
        }

        // flip side to play
        self.side_to_play = match self.side_to_play {
            Color::White => Color::Black,
            Color::Black => Color::White
        };
    }

    pub fn get_bitboard(&mut self, piece: Piece) -> &mut Bitboard {
        match piece.color {
            Color::White => {
                match piece.kind {
                    PieceKind::Pawn => &mut self.white.pawns,
                    PieceKind::Knight => &mut self.white.knights,
                    PieceKind::Bishop => &mut self.white.bishops,
                    PieceKind::Rook => &mut self.white.rooks,
                    PieceKind::Queen => &mut self.white.queens,
                    PieceKind::King => &mut self.white.king
                }
            },

            Color::Black => {
                match piece.kind {
                    PieceKind::Pawn => &mut self.black.pawns,
                    PieceKind::Knight => &mut self.black.knights,
                    PieceKind::Bishop => &mut self.black.bishops,
                    PieceKind::Rook => &mut self.black.rooks,
                    PieceKind::Queen => &mut self.black.queens,
                    PieceKind::King => &mut self.black.king
                }

            }
        }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let line = "+---+---+---+---+---+---+---+---+\n";
        try!(write!(f, "{}", line));

        for rank in (0..8).rev() {
            try!(write!(f, "|"));

            for file in 0..8 {
                let sq = Square::from_coords(file, rank);

                match self.piece_at(sq) {
                    Some(piece) => try!(write!(f, " {} |", piece)),
                    None => try!(write!(f, "   |"))
                };

            }

            try!(write!(f, "\n"));
            try!(write!(f, "{}", line));
        }

        try!(write!(f, "To play: {:?}\n", self.side_to_play));
        try!(write!(f, "En passant: {:?}\n", self.en_passant));
        try!(write!(f, "OO: {}, OOO: {}, oo: {}, ooo: {}\n",
                        self.white_can_oo, self.white_can_ooo,
                        self.black_can_oo, self.black_can_ooo));
        try!(write!(f, "Half-move: {}, Full-move: {}\n",
                        self.halfmove_clock, self.fullmove_number));

        Ok(())
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_write = match (&self.color, &self.kind) {
            (&Color::White, &PieceKind::Pawn) => 'P',
            (&Color::White, &PieceKind::Knight) => 'N',
            (&Color::White, &PieceKind::Bishop) => 'B',
            (&Color::White, &PieceKind::Rook) => 'R',
            (&Color::White, &PieceKind::Queen) => 'Q',
            (&Color::White, &PieceKind::King) => 'K',
            (&Color::Black, &PieceKind::Pawn) => 'p',
            (&Color::Black, &PieceKind::Knight) => 'n',
            (&Color::Black, &PieceKind::Bishop) => 'b',
            (&Color::Black, &PieceKind::Rook) => 'r',
            (&Color::Black, &PieceKind::Queen) => 'q',
            (&Color::Black, &PieceKind::King) => 'k'
        };

        write!(f, "{}", to_write)
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

impl Army {
    pub fn get_bitboard_mut(&mut self, kind: PieceKind) -> &mut Bitboard {
        match kind {
            PieceKind::Pawn => &mut self.pawns,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Queen => &mut self.queens,
            PieceKind::King => &mut self.king
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Move {
    from: Square,
    to: Square,
    promote_to: Option<PieceKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Piece {
    color: Color,
    kind: PieceKind
}

impl Piece {
    pub fn new(color: Color, kind: PieceKind) -> Piece {
        Piece {
            color: color,
            kind: kind
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Color {
    White,
    Black
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
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
    assert_eq!(Some(Square::from_san("e3")), position.en_passant);
    assert_eq!(0, position.halfmove_clock);
    assert_eq!(1, position.fullmove_number);
}

#[test]
fn piece_at() {
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let position = Position::from_fen(fen).unwrap();

    let white_rook = Piece::new(Color::White, PieceKind::Rook);
    assert_eq!(Some(white_rook), position.piece_at(Square::from_san("a1")));
}

#[test]
fn make_move_e2e4_e7e5() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut position = Position::from_fen(fen).unwrap();

    let motion = Move {
        from: Square::from_san("e2"),
        to: Square::from_san("e4"),
        promote_to: None
    };

    position.make_move(motion);

    let white_pawn = Piece::new(Color::White, PieceKind::Pawn);
    assert_eq!(Some(white_pawn), position.piece_at(Square::from_san("e4")));
    assert_eq!(None, position.piece_at(Square::from_san("e2")));
    assert_eq!(Some(Square::from_san("e3")), position.en_passant);

    let motion = Move {
        from: Square::from_san("e7"),
        to: Square::from_san("e5"),
        promote_to: None
    };

    position.make_move(motion);

    let black_pawn = Piece::new(Color::Black, PieceKind::Pawn);
    assert_eq!(Some(black_pawn), position.piece_at(Square::from_san("e5")));
    assert_eq!(None, position.piece_at(Square::from_san("e7")));
    assert_eq!(Some(Square::from_san("e6")), position.en_passant);

    assert_eq!(2, position.fullmove_number);
}

#[test]
fn make_move_capture() {
    let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 2";
    let mut position = Position::from_fen(fen).unwrap();

    let motion = Move {
        from: Square::from_san("f3"),
        to: Square::from_san("e5"),
        promote_to: None
    };

    position.make_move(motion);

    let white_knight = Piece::new(Color::White, PieceKind::Knight);
    assert_eq!(Some(white_knight), position.piece_at(Square::from_san("e5")));
    assert_eq!(None, position.piece_at(Square::from_san("f3")));
    assert_eq!(0, position.halfmove_clock);

    let motion = Move {
        from: Square::from_san("c6"),
        to: Square::from_san("e5"),
        promote_to: None
    };

    position.make_move(motion);

    let black_knight = Piece::new(Color::Black, PieceKind::Knight);
    assert_eq!(Some(black_knight), position.piece_at(Square::from_san("e5")));
    assert_eq!(None, position.piece_at(Square::from_san("c6")));
    assert_eq!(0, position.halfmove_clock);
}

#[test]
fn make_move_castle() {
    let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let mut position = Position::from_fen(fen).unwrap();

    let motion = Move {
        from: Square::from_san("e1"),
        to: Square::from_san("g1"),
        promote_to: None
    };

    position.make_move(motion);

    let white_king = Piece::new(Color::White, PieceKind::King);
    let white_rook = Piece::new(Color::White, PieceKind::Rook);
    assert_eq!(Some(white_king), position.piece_at(Square::from_san("g1")));
    assert_eq!(Some(white_rook), position.piece_at(Square::from_san("f1")));
    assert_eq!(None, position.piece_at(Square::from_san("h1")));
    assert!(!position.white_can_oo);
    assert!(!position.white_can_ooo);

    let motion = Move {
        from: Square::from_san("a8"),
        to: Square::from_san("a6"),
        promote_to: None
    };

    position.make_move(motion);

    assert!(position.black_can_oo);
    assert!(!position.black_can_ooo);
}

#[test]
fn make_move_promotion() {
    let fen = "8/7P/8/5K1k/8/8/8/8 w - - 0 1";
    let mut position = Position::from_fen(fen).unwrap();

    let motion = Move {
        from: Square::from_san("h7"),
        to: Square::from_san("h8"),
        promote_to: Some(PieceKind::Rook)
    };

    position.make_move(motion);

    let white_rook = Piece::new(Color::White, PieceKind::Rook);
    assert_eq!(Some(white_rook), position.piece_at(Square::from_san("h8")));
    assert_eq!(None, position.piece_at(Square::from_san("h7")));
}
