use super::helper::Sides;
use super::{Position, Board, Move};

pub fn get_all_moves(board: &Board, pos: &Position) -> Vec<Move> {
    vec![Move::new(Position::new(3, 1), Position::new(3, 3)), Move::new(Position::new(3, 1), Position::new(3, 2))]
}
