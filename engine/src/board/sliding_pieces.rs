use super::bitboard::BitBoard;
use super::helper::{Color, Piece};
use super::precompute::{DIRECTION_OFFSETS, NUM_SQUARES_TO_EDGE};
use super::{Board, Move, Position};

pub fn get_all_moves_bishop(board: &mut Board, pos: Position) {
    get_all_moves_sliding(board, pos, 4, 8);
}

pub fn get_all_attacks_bishop(board: &Board, pieces: BitBoard, color: Color) -> BitBoard {
    let empty = (!board.white_total & !board.black_total) | board.pieces[(!color, Piece::King)];
    nowe_dumb_7_fill(pieces, empty) |
    soea_dumb_7_fill(pieces, empty) |
    noea_dumb_7_fill(pieces, empty) |
    sowe_dumb_7_fill(pieces, empty)
}

pub fn get_all_moves_rook(board: &mut Board, pos: Position) {
    get_all_moves_sliding(board, pos, 0, 4);
}

pub fn get_all_attacks_rook(board: &Board, pieces: BitBoard, color: Color) -> BitBoard {
    let empty = (!board.white_total & !board.black_total) | board.pieces[(!color, Piece::King)];
    nort_dumb_7_fill(pieces, empty) |
    sout_dumb_7_fill(pieces, empty) |
    west_dumb_7_fill(pieces, empty) |
    east_dumb_7_fill(pieces, empty)
}

pub fn get_all_moves_queen(board: &mut Board, pos: Position) {
    get_all_moves_sliding(board, pos, 0, 8);
}

pub fn get_all_moves_sliding(
    board: &mut Board,
    pos: Position,
    start_dir: usize,
    end_dir: usize,
) {
    // when the king is in double-check, the king has to move
    if board.king_attacker_count > 1 {
        return;
    }

    let is_pinned = board.pinned_pieces.has(pos);
    // if king is in check, no pinned piece has a legal move
    if is_pinned && board.king_attacker_count != 0 {
        return;
    }
    let friendly_side = board.current_player();

    for dir_idx in start_dir..end_dir {
        let dir = DIRECTION_OFFSETS[dir_idx as usize] as i8;
        for n in 0..NUM_SQUARES_TO_EDGE[pos.0 as usize][dir_idx] {
            let target_square = Position((pos.0 as i8 + dir * (n + 1)) as u8);
            if board.king_attacker_count == 1
                && !(board.king_attacker_mask | board.king_attacker_block_mask).has(target_square)
            {
                if !board.tile_is_empty(target_square) {
                    break;
                } else {
                    continue;
                }
            }

            if is_pinned && !board.pinned_pieces_move_masks[pos.0 as usize].has(target_square) {
                break;
            }

            let target_square_is_empty = board.tile_is_empty(target_square);
            // target square is not empty and there is a friendly piece => stop search in this direction
            if !target_square_is_empty && board.piece_color_on_tile(target_square, friendly_side) {
                break;
            }

            board.move_list.push(Move::new(pos, target_square));

            // target square is not empty, therefore is enemy piece => stop search after adding
            // move to list
            if !target_square_is_empty {
                break;
            }
        }
    }
}

#[inline]
fn sout_dumb_7_fill(rook: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<-8, 0>(rook, empty)
}

#[inline]
fn nort_dumb_7_fill(rook: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<8, 0>(rook, empty)
}

#[inline]
fn east_dumb_7_fill(rook: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<1, 0xfefefefefefefefe>(rook, empty)
}

#[inline]
fn noea_dumb_7_fill(bishop: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<9, 0xfefefefefefefefe>(bishop, empty)
}

#[inline]
fn soea_dumb_7_fill(bishop: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<-7, 0xfefefefefefefefe>(bishop, empty)
}

#[inline]
fn west_dumb_7_fill(rook: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<-1, 0x7f7f7f7f7f7f7f7f>(rook, empty)
}

#[inline]
fn sowe_dumb_7_fill(bishop: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<-9, 0x7f7f7f7f7f7f7f7f>(bishop, empty)
}

#[inline]
fn nowe_dumb_7_fill(bishop: BitBoard, empty: BitBoard) -> BitBoard {
    dumb_7_fill::<7, 0x7f7f7f7f7f7f7f7f>(bishop, empty)
}

fn dumb_7_fill<const DIR: i8, const WRAP_MASK: u64>(mut pieces: BitBoard, mut empty: BitBoard) -> BitBoard {
    let mut flood = pieces;
    if WRAP_MASK != 0 {
        empty &= WRAP_MASK;
    }
    if DIR > 0 {
        let d = DIR as u8;
        pieces = (pieces << d) & empty; flood |= pieces;
        pieces = (pieces << d) & empty; flood |= pieces;
        pieces = (pieces << d) & empty; flood |= pieces;
        pieces = (pieces << d) & empty; flood |= pieces;
        pieces = (pieces << d) & empty; flood |= pieces;
        flood |= (pieces << d) & empty;
        if WRAP_MASK == 0 {
            flood << d
        } else {
            (flood << d) & WRAP_MASK
        }
    } else {
        let d = (DIR * -1) as u8;
        pieces = (pieces >> d) & empty; flood |= pieces;
        pieces = (pieces >> d) & empty; flood |= pieces;
        pieces = (pieces >> d) & empty; flood |= pieces;
        pieces = (pieces >> d) & empty; flood |= pieces;
        pieces = (pieces >> d) & empty; flood |= pieces;
        flood |= (pieces >> d) & empty;
        if WRAP_MASK == 0 {
            flood >> d
        } else {
            (flood >> d) & WRAP_MASK
        }
    }
}
