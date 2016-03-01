mod iter {
    use bitboard::Bitboard;
    use motion::Move;
    use square::Square;
    use position::{Color, Piece, PieceKind, Position};

    struct MovesIter<'a> {
        position: &'a Position,
        next_to_return: Option<Piece>,
        buffer: Vec<Move>,
    }

    impl<'a> Iterator for MovesIter<'a> {
        type Item = Move;

        fn next(&mut self) -> Option<Move> {
            if let Some(piece) = self.buffer.pop() {
                return Some(piece);
            }

            if let Some(next_to_return) = self.next_to_return {
                self.get_moves(next_to_return);
                self.next_to_return = self.next_piece(next_to_return);

                return self.buffer.pop();
            }

            None
        }
    }

    impl<'a> MovesIter<'a> {
        fn new(position: &Position) -> MovesIter {
            let next_to_return = Piece::new(position.side_to_play, PieceKind::Pawn);
            MovesIter {
                position: position,
                next_to_return: Some(next_to_return),
                buffer: vec![]
            }
        }

        fn get_moves(&mut self, piece: Piece) {
            match (piece.color, piece.kind) {
                (Color::White, PieceKind::Pawn) => self.get_white_pawn_moves(),
                (Color::Black, PieceKind::Pawn) => self.get_black_pawn_moves(),
                _ => panic!()
            };
        }

        fn next_piece(&self, piece: Piece) -> Option<Piece> {
            None
        }

        fn get_white_pawn_moves(&mut self) {
            let promote_pieces = [
                PieceKind::Knight,
                PieceKind::Bishop,
                PieceKind::Rook,
                PieceKind::Queen
            ];

            for square in self.position.white.pawns.squares() {
                let pawn_attacks = super::bitmask::white_pawn_attacks(square);
                let pawn_attacks = pawn_attacks & self.position.black.all;

                for attacked_square in pawn_attacks.squares() {
                    if attacked_square.rank() == 7 {

                        for promote_to in promote_pieces.iter() {
                            self.buffer.push(Move {
                                from: square,
                                to: attacked_square,
                                promote_to: Some(*promote_to),
                                castling: None
                            });
                        }
                    } else {
                        self.buffer.push(Move {
                            from: square,
                            to: attacked_square,
                            promote_to: None,
                            castling: None
                        });
                    }
                }

                if (self.position.all & (square + 8).to_bitboard()).is_empty() {
                    if square.rank() == 6 {
                        for promote_to in promote_pieces.iter() {
                            self.buffer.push(Move {
                                from: square,
                                to: square + 8,
                                promote_to: Some(*promote_to),
                                castling: None
                            });
                        }
                    } else {
                        self.buffer.push(Move {
                            from: square,
                            to: square + 8,
                            promote_to: None,
                            castling: None
                        });
                    }

                    if square.rank() == 1 {
                        let two_square = (square + 16).to_bitboard();

                        if (self.position.all & two_square).is_empty() {
                            self.buffer.push(Move {
                                from: square,
                                to: square + 16,
                                promote_to: None,
                                castling: None
                            });
                        }
                    }
                }
            }
        }

        fn get_black_pawn_moves(&mut self) {
            let promote_pieces = [
                PieceKind::Knight,
                PieceKind::Bishop,
                PieceKind::Rook,
                PieceKind::Queen
            ];

            for square in self.position.black.pawns.squares() {
                let pawn_attacks = super::bitmask::black_pawn_attacks(square);
                let pawn_attacks = pawn_attacks & self.position.white.all;

                for attacked_square in pawn_attacks.squares() {
                    if attacked_square.rank() == 0 {

                        for promote_to in promote_pieces.iter() {
                            self.buffer.push(Move {
                                from: square,
                                to: attacked_square,
                                promote_to: Some(*promote_to),
                                castling: None
                            });
                        }
                    } else {
                        self.buffer.push(Move {
                            from: square,
                            to: attacked_square,
                            promote_to: None,
                            castling: None
                        });
                    }
                }

                if (self.position.all & (square - 8).to_bitboard()).is_empty() {
                    if square.rank() == 1 {
                        for promote_to in promote_pieces.iter() {
                            self.buffer.push(Move {
                                from: square,
                                to: square - 8,
                                promote_to: Some(*promote_to),
                                castling: None
                            });
                        }
                    } else {
                        self.buffer.push(Move {
                            from: square,
                            to: square - 8,
                            promote_to: None,
                            castling: None
                        });
                    }

                    if square.rank() == 6 {
                        let two_square = (square - 16).to_bitboard();

                        if (self.position.all & two_square).is_empty() {
                            self.buffer.push(Move {
                                from: square,
                                to: square - 16,
                                promote_to: None,
                                castling: None
                            });
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_pawn_moves() {
        let fen = "4b3/p2P1p1p/1P6/5P2/5p2/p6p/1P1PpP1P/8 w - - 0 1";
        let position = Position::from_fen(fen).unwrap();

        // I have no good way to test this. See for youself, it's correct.
        let iter = MovesIter::new(&position);
        for motion in iter {
            println!("{}", motion.from.to_bitboard() | motion.to.to_bitboard());
        }
        // panic!();
    }
}

mod bitmask {
    use bitboard::Bitboard;
    use square::Square;

    pub fn white_pawn_attacks(square: Square) -> Bitboard {
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        let mut result = Bitboard::new(0);

        result = add_if_in_bounds(result, file + 1, rank + 1);
        result = add_if_in_bounds(result, file - 1, rank + 1);

        result
    }

    pub fn black_pawn_attacks(square: Square) -> Bitboard {
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        let mut result = Bitboard::new(0);

        result = add_if_in_bounds(result, file + 1, rank - 1);
        result = add_if_in_bounds(result, file - 1, rank - 1);

        result
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
}
