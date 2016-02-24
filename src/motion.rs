use square::Square;
use position::PieceKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promote_to: Option<PieceKind>,
    pub castling: Option<CastlingType>
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CastlingType {
    Kingside,
    Queenside
}
