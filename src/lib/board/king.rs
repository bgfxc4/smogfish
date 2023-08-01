use super::bitboard::BitBoard;
use super::helper::{Sides, Pieces};
use super::precompute::PRECOMPUTED_LOOKUPS;
use super::{Position, Board, Move};

pub fn get_all_moves_pseudolegal(board: &Board, pos: &Position, moves: &mut Vec<Move>) {
    let square_idx = (pos.row*8+pos.col) as i8;

    for dir_idx in 0 .. 8 {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;

        if (square_idx + dir) >= 64 || (square_idx + dir) < 0 {
            continue;
        } 

        let target_square = square_idx as i8 + dir;
        let target_piece = board.get_by_idx(target_square as u8);

        // target square is not empty and there is a friendly piece => stop search in this direction
        if (target_piece.0 != Pieces::EMPTY) && ((target_piece.1 == Sides::WHITE) == board.is_white_to_play()) {
            continue;
        }

        // the target square is in check
        if board.check_mask & BitBoard(1 << target_square) != BitBoard(0) {
            continue;
        }

        moves.push(Move::new(pos, &Position { row: (target_square/8) as u8, col: (target_square % 8) as u8 }));
    }
}

pub fn get_all_attacks(_board: &Board, pos: &Position) -> BitBoard {
    let mut ret = BitBoard(0);
    let square_idx = (pos.row*8+pos.col) as i8;

    for dir_idx in 0 .. 8 {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;

        if (square_idx + dir) >= 64 || (square_idx + dir) < 0 {
            continue;
        } 

        let target_square = square_idx as i8 + dir;
        ret |= BitBoard(1 << target_square);
    }
    ret
}
