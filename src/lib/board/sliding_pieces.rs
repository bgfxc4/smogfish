use super::bitboard::BitBoard;
use super::helper::{Color, Piece};
use super::precompute::PRECOMPUTED_LOOKUPS;
use super::{Board, Move, Position};

pub fn get_all_moves_bishop_pseudolegal(board: &Board, pos: Position, moves: &mut Vec<Move>) {
    get_all_moves_sliding_pseudolegal(board, pos, moves, 4, 8);
}

pub fn get_all_attacks_bishop(board: &Board, pos: Position, color: Color) -> BitBoard {
    get_all_attacks_sliding(board, pos, color, 4, 8)
}

pub fn get_all_moves_rook_pseudolegal(board: &Board, pos: Position, moves: &mut Vec<Move>) {
    get_all_moves_sliding_pseudolegal(board, pos, moves, 0, 4);
}

pub fn get_all_attacks_rook(board: &Board, pos: Position, color: Color) -> BitBoard {
    get_all_attacks_sliding(board, pos, color, 0, 4)
}

pub fn get_all_moves_queen_pseudolegal(board: &Board, pos: Position, moves: &mut Vec<Move>) {
    get_all_moves_sliding_pseudolegal(board, pos, moves, 0, 8);
}

pub fn get_all_attacks_queen(board: &Board, pos: Position, color: Color) -> BitBoard {
    get_all_attacks_sliding(board, pos, color, 0, 8)
}

pub fn get_all_moves_sliding_pseudolegal(
    board: &Board,
    pos: Position,
    moves: &mut Vec<Move>,
    start_dir: usize,
    end_dir: usize,
) {
    // when the king is in double-check, the king has to move
    if board.king_attacker_count > 1 {
        return;
    }

    let is_pinned = board.pinned_pieces & BitBoard(1 << pos) != BitBoard(0);
    // if king is in check, no pinned piece has a legal move
    if is_pinned && board.king_attacker_count != 0 {
        return;
    }
    let friendly_side = if board.is_white_to_play() {
        Color::White
    } else {
        Color::Black
    };

    for dir_idx in start_dir..end_dir {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;
        for n in 0..PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[pos as usize][dir_idx] {
            let target_square = (pos as i8 + dir * (n + 1)) as u8;
            if board.king_attacker_count == 1
                && (board.king_attacker_mask | board.king_attacker_block_mask)
                    & BitBoard(1 << target_square)
                    == BitBoard(0)
            {
                if !board.tile_is_empty(target_square) {
                    break;
                } else {
                    continue;
                }
            }

            if is_pinned
                && board.pinned_pieces_move_mask & BitBoard(1 << target_square) == BitBoard(0)
            {
                break;
            }

            let target_square_is_empty = board.tile_is_empty(target_square);
            // target square is not empty and there is a friendly piece => stop search in this direction
            if !target_square_is_empty && board.piece_color_on_tile(target_square, friendly_side) {
                break;
            }

            moves.push(Move::new(pos, target_square));

            // target square is not empty, therefore is enemy piece => stop search after adding
            // move to list
            if !target_square_is_empty {
                break;
            }
        }
    }
}

pub fn get_all_attacks_sliding(
    board: &Board,
    pos: Position,
    color: Color,
    start_dir: usize,
    end_dir: usize,
) -> BitBoard {
    let mut ret = BitBoard(0);

    for dir_idx in start_dir..end_dir {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;
        for n in 0..PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[pos as usize][dir_idx] {
            let target_square = (pos as i8 + dir * (n + 1)) as u8;
            ret |= BitBoard(1 << target_square);

            // stop search if there is a piece blocking the line, but continue if it is the enemy
            // king, because he should not be included in this search
            if !board.tile_is_empty(target_square)
                && !board.piece_is_type(target_square, color, Piece::King)
            {
                break;
            }
        }
    }
    ret
}
