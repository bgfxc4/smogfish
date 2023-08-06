use super::bitboard::BitBoard;
use super::helper::Color;
use super::{Board, Move, Position};

pub fn get_all_moves_pseudolegal(board: &mut Board, pos: Position) {
    // when the king is in double-check, the king has to move
    if board.king_attacker_count > 1 {
        return;
    }

    let attacker_and_block_mask = if board.king_attacker_count == 0 {
        BitBoard(0)
    } else {
        board.king_attacker_block_mask | board.king_attacker_mask
    };

    let active = board.current_player();
    let enemy_side = !active;
    let modi: i8 = match active {
        Color::White => 1,
        Color::Black => -1,
    };
    let en_passant = board.get_en_passant();
    let is_pinned = board.pinned_pieces & BitBoard(1 << pos) != BitBoard(0);

    // move one forward
    let p = (pos as i8 + 8 * modi) as u8;
    if (board.king_attacker_count != 1 || attacker_and_block_mask & BitBoard(1 << p) != BitBoard(0))
        && (!is_pinned || board.pinned_pieces_move_mask & BitBoard(1 << p) != BitBoard(0))
    {
        if board.tile_is_empty(p) {
            let is_promotion = match active {
                Color::White => p / 8 == 7,
                Color::Black => p / 8 == 0,
            };
            if is_promotion {
                board.move_list.push(Move::new_with_flags(pos, p, 5));
                board.move_list.push(Move::new_with_flags(pos, p, 6));
                board.move_list.push(Move::new_with_flags(pos, p, 7));
                board.move_list.push(Move::new_with_flags(pos, p, 8));
            } else {
                board.move_list.push(Move::new(pos, p));
            }
        }
    }

    // move two forward
    let p = (pos as i8 + 8 * modi * 2) as u8;
    let is_in_start_pos = (modi == 1 && pos / 8 == 1) || (modi == -1 && pos / 8 == 6);
    if is_in_start_pos
        && (board.king_attacker_count != 1
            || attacker_and_block_mask & BitBoard(1 << p) != BitBoard(0))
        && (!is_pinned || board.pinned_pieces_move_mask & BitBoard(1 << p) != BitBoard(0))
    {
        if board.tile_is_empty(p) && board.tile_is_empty((pos as i8 + 8 * modi) as u8) {
            board.move_list.push(Move::new_with_flags(pos, p, 2));
        }
    }

    // take left and right
    for (file, dir) in [(7, 1), (0, -1)] {
        if pos % 8 != file {
            let p = (pos as i8 + 8 * modi + dir) as u8;
            if board.king_attacker_count != 1
                || attacker_and_block_mask & BitBoard(1 << p) != BitBoard(0)
                    && (!is_pinned
                        || board.pinned_pieces_move_mask & BitBoard(1 << p) != BitBoard(0))
            {
                if board.piece_color_on_tile(p, enemy_side) && !board.tile_is_empty(p) {
                    let is_promotion = match active {
                        Color::White => p / 8 == 7,
                        Color::Black => p / 8 == 0,
                    };
                    if is_promotion {
                        board.move_list.push(Move::new_with_flags(pos, p, 5));
                        board.move_list.push(Move::new_with_flags(pos, p, 6));
                        board.move_list.push(Move::new_with_flags(pos, p, 7));
                        board.move_list.push(Move::new_with_flags(pos, p, 8));
                    } else {
                        board.move_list.push(Move::new(pos, p));
                    }
                } else if match active {
                    Color::White => (p / 8 == 5) && (en_passant == (p % 8) as u16),
                    Color::Black => (p / 8 == 2) && (en_passant == (p % 8) as u16),
                } {
                    if board.en_passant_pinned_piece == 65 {
                        // there can only be one en passant on the
                        // board => if one en passant piece is
                        // pinned, it is this one
                        board.move_list.push(Move::new_with_flags(pos, p, 1));
                    }
                }
            }
        }
    }
}

pub fn get_all_attacks(board: &Board, pos: Position) -> BitBoard {
    let mut ret = BitBoard(0);
    let modi: i8 = match board.current_player() {
        Color::White => -1,
        Color::Black => 1,
    };

    // take right
    if pos % 8 != 7 {
        // check for piece on target square not necessary here
        let p = (pos as i8 + 8 * modi + 1) as u8;
        ret |= BitBoard(1 << p);
    }

    // take left
    if pos % 8 != 0 {
        let p = (pos as i8 + 8 * modi - 1) as u8;
        ret |= BitBoard(1 << p);
    }
    ret
}
