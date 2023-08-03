use super::bitboard::BitBoard;
use super::precompute::PRECOMPUTED_LOOKUPS;
use super::{Position, Board, Move};

pub fn get_all_moves(board: &Board, pos: &Position, moves: &mut Vec<Move>) {
    // when the king is in double-check, the king has to move
    if board.king_attacker_count > 1 {
        return
    }
    let square_index = (pos.row*8 + pos.col) as i8;
    let is_pinned = board.pinned_pieces & BitBoard(1 << square_index) != BitBoard(0);
    if is_pinned {
        return
    }

    let mut attack_mask = PRECOMPUTED_LOOKUPS.KNIGHT_ATTACKS[square_index as usize];
    
    if board.is_white_to_play() {
        attack_mask &= !board.white_total;
    } else {
        attack_mask &= !board.black_total;
    }
    
    if board.king_attacker_count == 1 {
        attack_mask &= board.king_attacker_block_mask | board.king_attacker_mask;
    }

    for target_square in 0..64 {
        if attack_mask & BitBoard(1 << target_square) != BitBoard(0) {
            moves.push(Move::new(pos, &Position { row: (target_square/8) as u8, col: (target_square % 8) as u8 }));
        }
    }
}

pub fn get_all_attacks(_board: &Board, pos: &Position) -> BitBoard {
    let square_index = (pos.row*8 + pos.col) as i8;
    let attack_mask = PRECOMPUTED_LOOKUPS.KNIGHT_ATTACKS[square_index as usize];
    attack_mask
}
