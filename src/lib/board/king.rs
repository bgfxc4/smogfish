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

// the enemy king can be ignored here, he can not be an attacker of the own king
pub fn calc_king_attacker_masks(board: &mut Board, pos: u8) {
    let (friendly_side, enemy_side) = if board.is_white_to_play() { (Sides::WHITE, Sides::BLACK) } else { (Sides::BLACK, Sides::WHITE) };

    let enemy_knight_attackers = board.pieces[enemy_side as usize][Pieces::KNIGHT as usize] & PRECOMPUTED_LOOKUPS.KNIGHT_ATTACKS[pos as usize];

    let enemy_pawn_attackers = board.pieces[enemy_side as usize][Pieces::PAWN as usize] & PRECOMPUTED_LOOKUPS.KING_PAWN_ATTACKS[friendly_side as usize][pos as usize];

    let mut enemy_sliding_piece_attackers = BitBoard(0);
    board.king_attacker_block_mask = BitBoard(0);
    for dir_idx in 0..8 {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;

        let mut temp_block_mask = BitBoard(0);
        for n in 0..PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[pos as usize][dir_idx] {

            let target_square = pos as i8 + dir * (n+1);
            let target_piece = board.get_by_idx(target_square as u8);

            // target square is not empty and there is a friendly piece => stop search in this direction
            if (target_piece.0 != Pieces::EMPTY) && (target_piece.1 == friendly_side) {
                break;
            }

            // target square is not empty, therefore is enemy piece => stop search after adding
            // piece to mask (only if the piece that is found can attack in this direction)
            if target_piece.0 != Pieces::EMPTY {

                if target_piece.0 == Pieces::QUEEN || (target_piece.0 == Pieces::ROOK && dir_idx < 4) || (target_piece.0 == Pieces::BISHOP && dir_idx >= 4) {
                    enemy_sliding_piece_attackers |= BitBoard(1 << target_square);
                    // if there is a sliding piece attacking the king, add all squares from the
                    // king to the sliding piece to the block mask
                    board.king_attacker_block_mask |= temp_block_mask;
                }
                break;
            }
            
            // if there is no piece on the square, add the square to the temp. mask
            temp_block_mask |= BitBoard(1 << target_square);
        }
    }

    board.king_attacker_mask = enemy_knight_attackers | enemy_pawn_attackers | enemy_sliding_piece_attackers;
}
