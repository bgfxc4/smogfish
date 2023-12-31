use super::bitboard::BitBoard;
use super::helper::Color;
use super::{Board, Move, Position};

pub fn get_all_moves(board: &mut Board, pos: Position) {
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
    let is_pinned = board.pinned_pieces.has(pos);

    // move one forward
    let p = Position((pos.0 as i8 + 8 * modi) as u8);
    if (board.king_attacker_count != 1 || attacker_and_block_mask.has(p))
        && (!is_pinned || board.pinned_pieces_move_masks[pos.0 as usize].has(p))
    {
        if board.tile_is_empty(p) {
            let is_promotion = match active {
                Color::White => p.rank() == 7,
                Color::Black => p.rank() == 0,
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
    let p = Position((pos.0 as i8 + 8 * modi * 2) as u8);
    let is_in_start_pos = (modi == 1 && pos.rank() == 1) || (modi == -1 && pos.rank() == 6);
    if is_in_start_pos
        && (board.king_attacker_count != 1 || attacker_and_block_mask.has(p))
        && (!is_pinned || board.pinned_pieces_move_masks[pos.0 as usize].has(p))
    {
        if board.tile_is_empty(p) && board.tile_is_empty(Position((pos.0 as i8 + 8 * modi) as u8)) {
            board.move_list.push(Move::new_with_flags(pos, p, 2));
        }
    }

    // take left and right
    for (file, dir) in [(7, 1), (0, -1)] {
        if pos.file() != file {
            let p = Position((pos.0 as i8 + 8 * modi + dir) as u8);
            if (board.king_attacker_count != 1
                || attacker_and_block_mask.has(p))
                    && (!is_pinned || board.pinned_pieces_move_masks[pos.0 as usize].has(p))
            {
                if board.piece_color_on_tile(p, enemy_side) && !board.tile_is_empty(p) {
                    let is_promotion = match active {
                        Color::White => p.rank() == 7,
                        Color::Black => p.rank() == 0,
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
                    Color::White => (p.rank() == 5) && (en_passant == (p.file()) as u16),
                    Color::Black => (p.rank() == 2) && (en_passant == (p.file()) as u16),
                } {
                    if board.en_passant_pinned_piece == 65 {
                        // there can only be one en passant on the
                        // board => if one en passant piece is
                        // pinned, it is this one
                        board.move_list.push(Move::new_with_flags(pos, p, 1));
                    }
                }
            } else if board.king_attacker_count == 1 && board.king_attacker_mask.has(Position((pos.0 as i8 + dir) as u8)) && p.file() as u16 == en_passant {
                // if a pawn moved two squares forwards and is checking the king, it can be taken
                // en passant
                if match active {
                    Color::White => p.rank() == 5,
                    Color::Black => p.rank() == 2,
                } {
                    board.move_list.push(Move::new_with_flags(pos, p, 1));
                }
            }
        }
    }
}

pub fn get_all_attacks(board: &Board, pieces: BitBoard) -> BitBoard {

    if board.current_player() == Color::White {
        // get attacks of black pawns
        let not_a_mask = 0xfefefefefefefefe;
        let not_h_mask = 0x7f7f7f7f7f7f7f7f;
        ((pieces >> 7) & not_a_mask) | ((pieces >> 9) & not_h_mask)
    } else {
        let not_a_mask = 0xfefefefefefefefe;
        let not_h_mask = 0x7f7f7f7f7f7f7f7f;
        ((pieces << 9) & not_a_mask) | ((pieces << 7) & not_h_mask)
    }
}
