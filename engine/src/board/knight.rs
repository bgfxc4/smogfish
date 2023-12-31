use super::bitboard::BitBoard;
use super::helper::Color;
use super::precompute::KNIGHT_ATTACKS;
use super::{Board, Move, Position};

pub fn get_all_moves(board: &mut Board, pos: Position) {
    // when the king is in double-check, the king has to move
    if board.king_attacker_count > 1 {
        return;
    }
    let is_pinned = board.pinned_pieces.has(pos);
    if is_pinned {
        return;
    }

    let mut attack_mask = KNIGHT_ATTACKS[pos.0 as usize];

    match board.current_player() {
        Color::White => attack_mask &= !board.white_total,
        Color::Black => attack_mask &= !board.black_total,
    }

    if board.king_attacker_count == 1 {
        attack_mask &= board.king_attacker_block_mask | board.king_attacker_mask;
    }

    for i in attack_mask {
        board.move_list.push(Move::new(pos, i));
    }
}

pub fn get_all_attacks(_board: &Board, pos: Position) -> BitBoard {
    let attack_mask = KNIGHT_ATTACKS[pos.0 as usize];
    attack_mask
}
