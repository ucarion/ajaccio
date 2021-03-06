use std::fmt;

use fen;
use bitboard::Bitboard;
use square::Square;
use motion::{CastlingType, Move};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Position {
    pub white: Army,
    pub black: Army,
    pub all: Bitboard,

    pub side_to_play: Color,
    pub white_can_oo: bool,
    pub white_can_ooo: bool,
    pub black_can_oo: bool,
    pub black_can_ooo: bool,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u64,
    pub fullmove_number: u64
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

        position.update_special_bitboards();

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

    pub fn make_move(&mut self, motion: Move) -> UndoContext {
        let from = self.piece_at(motion.from).unwrap();
        let captured = self.piece_at(motion.to);

        let mut undo = UndoContext {
            halfmove_clock: self.halfmove_clock,
            captured: captured.map(|piece| piece.kind),
            en_passant: self.en_passant,
            reset_oo: false,
            reset_ooo: false
        };

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
        match self.side_to_play {
            Color::Black => {
                self.fullmove_number += 1;
            },

            _ => {}
        }

        if let Some(promote_to) = motion.promote_to {
            let army = match self.side_to_play {
                Color::White => &mut self.white,
                Color::Black => &mut self.black
            };

            let pawn_bitmask = motion.from.to_bitboard();
            army.pawns = army.pawns ^ pawn_bitmask;

            let promo_bitboard = army.get_bitboard_mut(promote_to);
            let promo_bitmask = motion.to.to_bitboard();
            *promo_bitboard = promo_bitboard.clone() | promo_bitmask;
        } else {
            // change the bitboard of the moving piece
            let bitboard = self.get_bitboard_mut(from.clone());
            let bitmask = motion.from.to_bitboard() | motion.to.to_bitboard();

            *bitboard = bitboard.clone() ^ bitmask;
        }

        // change the bitboard of any piece being captured
        match captured {
            Some(to) => {
                self.halfmove_clock = 0;

                let bitboard = self.get_bitboard_mut(to);
                let bitmask = motion.to.to_bitboard();

                *bitboard = bitboard.clone() ^ bitmask;
            },

            None => {}
        };

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
            },

            PieceKind::Rook => {
                // handle updating castling rights
                match self.side_to_play {
                    Color::White => {
                        if motion.from == Square::from_san("h1") {
                            self.white_can_oo = false;
                            undo.reset_oo = true;
                        }

                        if motion.from == Square::from_san("a1") {
                            self.white_can_ooo = false;
                            undo.reset_ooo = true;
                        }
                    },

                    Color::Black => {
                        if motion.from == Square::from_san("h8") {
                            self.black_can_oo = false;
                            undo.reset_oo = true;
                        }

                        if motion.from == Square::from_san("a8") {
                            self.black_can_ooo = false;
                            undo.reset_ooo = true;
                        }
                    }
                }
            },

            PieceKind::King => {
                self.white_can_oo = false;
                self.white_can_ooo = false;
                undo.reset_oo = true;
                undo.reset_ooo = true;

                if let Some(castling_type) = motion.castling {
                    match self.side_to_play {
                        Color::White => {
                            let bitmask = match castling_type {
                                CastlingType::Kingside => Square::from_san("h1").to_bitboard()
                                        | Square::from_san("f1").to_bitboard(),
                                CastlingType::Queenside => Square::from_san("a1").to_bitboard()
                                        | Square::from_san("d1").to_bitboard()
                            };

                            self.white.rooks = self.white.rooks ^ bitmask;
                        },

                        Color::Black => {
                            let bitmask = match castling_type {
                                CastlingType::Kingside => Square::from_san("h8").to_bitboard()
                                    | Square::from_san("f8").to_bitboard(),
                                CastlingType::Queenside => Square::from_san("a8").to_bitboard()
                                        | Square::from_san("d8").to_bitboard()
                            };

                            self.black.rooks = self.black.rooks ^ bitmask;
                        }
                    }
                }
            },

            _ => {}
        }

        // flip side to play
        self.side_to_play = match self.side_to_play {
            Color::White => Color::Black,
            Color::Black => Color::White
        };

        undo
    }

    pub fn undo_move(&mut self, motion: Move, undo: UndoContext) {
        let to = self.piece_at(motion.to).unwrap();

        if let Some(promote_to) = motion.promote_to {
            let army = match self.side_to_play {
                Color::White => &mut self.black,
                Color::Black => &mut self.white
            };

            let pawn_bitmask = motion.from.to_bitboard();
            army.pawns = army.pawns ^ pawn_bitmask;

            let promo_bitboard = army.get_bitboard_mut(promote_to);
            let promo_bitmask = motion.to.to_bitboard();

            *promo_bitboard = promo_bitboard.clone() ^ promo_bitmask;
        } else {
            // change the bitboard of the moving piece
            let bitboard = self.get_bitboard_mut(to.clone());
            let bitmask = motion.from.to_bitboard() | motion.to.to_bitboard();

            *bitboard = bitboard.clone() ^ bitmask;
        }

        if let Some(captured) = undo.captured {
            let side = self.side_to_play;
            let bitboard = self.get_army_mut(side).get_bitboard_mut(captured);
            let bitmask = motion.to.to_bitboard();

            *bitboard = bitboard.clone() ^ bitmask;
        };

        if undo.reset_oo {
            match self.side_to_play {
                Color::White => self.black_can_oo = true,
                Color::Black => self.white_can_oo = true
            };
        }

        if undo.reset_ooo {
            match self.side_to_play {
                Color::White => self.black_can_ooo = true,
                Color::Black => self.white_can_ooo = true
            };
        }

        if let Some(castling_type) = motion.castling {
            match self.side_to_play {
                Color::White => {
                    let bitmask = match castling_type {
                        CastlingType::Kingside => Square::from_san("h8").to_bitboard()
                            | Square::from_san("f8").to_bitboard(),
                        CastlingType::Queenside => Square::from_san("a8").to_bitboard()
                                | Square::from_san("d8").to_bitboard()
                    };

                    self.black.rooks = self.black.rooks ^ bitmask;
                },

                Color::Black => {
                    let bitmask = match castling_type {
                        CastlingType::Kingside => Square::from_san("h1").to_bitboard()
                                | Square::from_san("f1").to_bitboard(),
                        CastlingType::Queenside => Square::from_san("a1").to_bitboard()
                                | Square::from_san("d1").to_bitboard()
                    };

                    self.white.rooks = self.white.rooks ^ bitmask;
                }
            }
        }

        // restore state from the UndoContext
        self.halfmove_clock = undo.halfmove_clock;
        self.en_passant = undo.en_passant;

        match self.side_to_play {
            Color::White => { self.fullmove_number -= 1 },
            _ => {}
        }


        // flip side to play
        self.side_to_play = match self.side_to_play {
            Color::White => Color::Black,
            Color::Black => Color::White
        };
    }

    pub fn get_army_mut(&mut self, color: Color) -> &mut Army {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black
        }
    }

    pub fn get_bitboard_mut(&mut self, piece: Piece) -> &mut Bitboard {
        self.get_army_mut(piece.color).get_bitboard_mut(piece.kind)
    }

    fn update_special_bitboards(&mut self) {
        self.white.update_union();
        self.black.update_union();
        self.all = self.white.all | self.black.all;
    }
}

pub struct UndoContext {
    pub halfmove_clock: u64,
    pub captured: Option<PieceKind>,
    pub en_passant: Option<Square>,
    pub reset_oo: bool,
    pub reset_ooo: bool
}

impl fmt::Display for Position {
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Army {
    pub pawns: Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks: Bitboard,
    pub queens: Bitboard,
    pub king: Bitboard,
    pub all: Bitboard
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

    pub fn update_union(&mut self) {
        self.all =
                self.pawns | self.knights | self.bishops | self.rooks | self.queens | self.king;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind
}

impl Piece {
    pub fn new(color: Color, kind: PieceKind) -> Piece {
        Piece {
            color: color,
            kind: kind
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    White,
    Black
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        promote_to: None,
        castling: None
    };

    position.make_move(motion);

    let white_pawn = Piece::new(Color::White, PieceKind::Pawn);
    assert_eq!(Some(white_pawn), position.piece_at(Square::from_san("e4")));
    assert_eq!(None, position.piece_at(Square::from_san("e2")));
    assert_eq!(Some(Square::from_san("e3")), position.en_passant);

    let motion = Move {
        from: Square::from_san("e7"),
        to: Square::from_san("e5"),
        promote_to: None,
        castling: None
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
        promote_to: None,
        castling: None
    };

    position.make_move(motion);

    let white_knight = Piece::new(Color::White, PieceKind::Knight);
    assert_eq!(Some(white_knight), position.piece_at(Square::from_san("e5")));
    assert_eq!(None, position.piece_at(Square::from_san("f3")));
    assert_eq!(0, position.halfmove_clock);

    let motion = Move {
        from: Square::from_san("c6"),
        to: Square::from_san("e5"),
        promote_to: None,
        castling: None
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
        promote_to: None,
        castling: Some(CastlingType::Kingside)
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
        promote_to: None,
        castling: None
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
        promote_to: Some(PieceKind::Rook),
        castling: None
    };

    position.make_move(motion);

    let white_rook = Piece::new(Color::White, PieceKind::Rook);
    assert_eq!(Some(white_rook), position.piece_at(Square::from_san("h8")));
    assert_eq!(None, position.piece_at(Square::from_san("h7")));
}

#[test]
fn make_ummake_moves() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut position = Position::from_fen(fen).unwrap();
    let original1 = position.clone();

    let motion1 = Move {
        from: Square::from_san("e2"),
        to: Square::from_san("e4"),
        promote_to: None,
        castling: None
    };

    let undo1 = position.make_move(motion1);
    let original2 = position.clone();

    let motion2 = Move {
        from: Square::from_san("e7"),
        to: Square::from_san("e5"),
        promote_to: None,
        castling: None
    };

    let undo2 = position.make_move(motion2);

    position.undo_move(motion2, undo2);
    assert_eq!(original2, position);
    position.undo_move(motion1, undo1);
    assert_eq!(original1, position);
}

#[test]
fn make_unmake_capture() {
    let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 2";
    let mut position = Position::from_fen(fen).unwrap();
    let original = position.clone();

    let motion = Move {
        from: Square::from_san("f3"),
        to: Square::from_san("e5"),
        promote_to: None,
        castling: None
    };

    let undo = position.make_move(motion);
    position.undo_move(motion, undo);

    assert_eq!(original, position);
}

#[test]
fn make_unmake_castle() {
    let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let mut position = Position::from_fen(fen).unwrap();
    let original = position.clone();

    let motion = Move {
        from: Square::from_san("e1"),
        to: Square::from_san("g1"),
        promote_to: None,
        castling: Some(CastlingType::Kingside)
    };

    let undo = position.make_move(motion);
    position.undo_move(motion, undo);

    assert_eq!(original, position);
}

#[test]
fn make_unmake_promotion() {
    let fen = "8/7P/8/5K1k/8/8/8/8 w - - 0 1";
    let mut position = Position::from_fen(fen).unwrap();
    let original = position.clone();

    let motion = Move {
        from: Square::from_san("h7"),
        to: Square::from_san("h8"),
        promote_to: Some(PieceKind::Rook),
        castling: None
    };

    let undo = position.make_move(motion);
    position.undo_move(motion, undo);

    assert_eq!(original, position);
}
