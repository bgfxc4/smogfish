use super::bitboard::BitBoard;
use super::helper::{Sides, Pieces};
use super::precompute::PRECOMPUTED_LOOKUPS;
use super::{Position, Board, Move};

pub fn get_all_moves_bishop_pseudolegal(board: &Board, pos: &Position, moves: &mut Vec<Move>) {
    get_all_moves_sliding_pseudolegal(board, pos, moves, 4, 8);
}

pub fn get_all_attacks_bishop(board: &Board, pos: &Position, color: u8) -> BitBoard {
    get_all_attacks_sliding(board, pos, color, 4, 8)
}

pub fn get_all_moves_rook_pseudolegal(board: &Board, pos: &Position, moves: &mut Vec<Move>) {
    get_all_moves_sliding_pseudolegal(board, pos, moves, 0, 4);
}

pub fn get_all_attacks_rook(board: &Board, pos: &Position, color: u8) -> BitBoard {
    get_all_attacks_sliding(board, pos, color, 0, 4)
}

pub fn get_all_moves_queen_pseudolegal(board: &Board, pos: &Position, moves: &mut Vec<Move>) {
    get_all_moves_sliding_pseudolegal(board, pos, moves, 0, 8);
}

pub fn get_all_attacks_queen(board: &Board, pos: &Position, color: u8) -> BitBoard {
    get_all_attacks_sliding(board, pos, color, 0, 8)
}

pub fn get_all_moves_sliding_pseudolegal(board: &Board, pos: &Position, moves: &mut Vec<Move>, start_dir: usize, end_dir: usize) {
    let square_idx = (pos.row*8+pos.col) as usize;

    for dir_idx in start_dir .. end_dir {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;
        for n in 0..PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[square_idx][dir_idx] {

            let target_square = square_idx as i8 + dir * (n+1);
            let target_piece = board.get_by_idx(target_square as u8);

            // target square is not empty and there is a friendly piece => stop search in this direction
            if (target_piece.0 != Pieces::EMPTY) && ((target_piece.1 == Sides::WHITE) == board.is_white_to_play()) {
                break;
            }

            moves.push(Move::new(pos, &Position { row: (target_square/8) as u8, col: (target_square % 8) as u8 }));

            // target square is not empty, therefore is enemy piece => stop search after adding
            // move to list 
            if target_piece.0 != Pieces::EMPTY {
                break;
            }
        }
    }
}

pub fn get_all_attacks_sliding(board: &Board, pos: &Position, color: u8, start_dir: usize, end_dir: usize) -> BitBoard {
    let mut ret = BitBoard(0);
    let square_idx = (pos.row*8+pos.col) as usize;

    for dir_idx in start_dir .. end_dir {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;
        for n in 0..PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[square_idx][dir_idx] {

            let target_square = square_idx as i8 + dir * (n+1);
            let target_piece = board.get_by_idx(target_square as u8);

            ret |= BitBoard(1 << target_square);

            // stop search if there is a piece blocking the line, but continue if it is the enemy
            // king, because he should not be included in this search 
            if (target_piece.0 != Pieces::EMPTY) && !(target_piece.0 == Pieces::KING && target_piece.1 != color) {
                break;
            }
        }
    }
    ret
}
