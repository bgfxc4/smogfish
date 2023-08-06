use super::bitboard::BitBoard;
use super::helper::{Pieces, Sides};
use super::precompute::PRECOMPUTED_LOOKUPS;
use super::{Board, Move, Position};

pub fn get_all_moves_pseudolegal(board: &Board, pos: Position, moves: &mut Vec<Move>) {
    let white_to_play = board.is_white_to_play();
    let (friendly_side, _enemy_side) = if white_to_play {
        (Sides::WHITE, Sides::BLACK)
    } else {
        (Sides::BLACK, Sides::WHITE)
    };

    for dir_idx in 0..8 {
        if PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[pos as usize][dir_idx] == 0 {
            continue;
        }
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;

        if (pos as i8 + dir) >= 64 || (pos as i8 + dir) < 0 {
            continue;
        }

        let target_square = (pos as i8 + dir) as u8;

        // target square is not empty and there is a friendly piece => stop search in this direction
        if !board.tile_is_empty(target_square)
            && board.piece_color_on_tile(target_square, friendly_side)
        {
            continue;
        }

        // the target square is in check
        if board.check_mask & BitBoard(1 << target_square) != BitBoard(0) {
            continue;
        }

        moves.push(Move::new(pos, target_square));
    }

    let total_castle_mask = board.white_total | board.black_total | board.check_mask;
    if white_to_play {
        if board.castle_white_short()
            && total_castle_mask & PRECOMPUTED_LOOKUPS.KING_CASTLE_CHECKS[Sides::WHITE as usize][0]
                == BitBoard(0)
        {
            moves.push(Move::new_with_flags(pos, 6 + 0 * 8, 3));
        }

        if board.castle_white_long()
            && total_castle_mask & PRECOMPUTED_LOOKUPS.KING_CASTLE_CHECKS[Sides::WHITE as usize][1]
                == BitBoard(0)
        {
            moves.push(Move::new_with_flags(pos, 2 + 0 * 8, 4));
        }
    } else {
        if board.castle_black_short()
            && total_castle_mask & PRECOMPUTED_LOOKUPS.KING_CASTLE_CHECKS[Sides::BLACK as usize][0]
                == BitBoard(0)
        {
            moves.push(Move::new_with_flags(pos, 6 + 7 * 8, 3));
        }

        if board.castle_black_long()
            && total_castle_mask & PRECOMPUTED_LOOKUPS.KING_CASTLE_CHECKS[Sides::BLACK as usize][1]
                == BitBoard(0)
        {
            moves.push(Move::new_with_flags(pos, 2 + 7 * 8, 4));
        }
    }
}

pub fn get_all_attacks(_board: &Board, pos: Position) -> BitBoard {
    let mut ret = BitBoard(0);

    for dir_idx in 0..8 {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;

        if (pos as i8 + dir) >= 64 || (pos as i8 + dir) < 0 {
            continue;
        }

        let target_square = pos as i8 + dir;
        ret |= BitBoard(1 << target_square);
    }
    ret
}

// the enemy king can be ignored here, he can not be an attacker of the own king
pub fn calc_king_attacker_masks(board: &mut Board, pos: u8) {
    let (friendly_side, enemy_side) = if board.is_white_to_play() {
        (Sides::WHITE, Sides::BLACK)
    } else {
        (Sides::BLACK, Sides::WHITE)
    };

    let enemy_knight_attackers = board.pieces[enemy_side as usize][Pieces::KNIGHT as usize]
        & PRECOMPUTED_LOOKUPS.KNIGHT_ATTACKS[pos as usize];

    let enemy_pawn_attackers = board.pieces[enemy_side as usize][Pieces::PAWN as usize]
        & PRECOMPUTED_LOOKUPS.KING_PAWN_ATTACKS[friendly_side as usize][pos as usize];

    let mut enemy_sliding_piece_attackers = BitBoard(0);
    board.king_attacker_block_mask = BitBoard(0);
    for dir_idx in 0..8 {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;

        let mut temp_block_mask = BitBoard(0);
        for n in 0..PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[pos as usize][dir_idx] {
            let target_square = (pos as i8 + dir * (n + 1)) as u8;
            let target_empty = board.tile_is_empty(target_square);

            // target square is not empty and there is a friendly piece => stop search in this direction
            if !target_empty && board.piece_color_on_tile(target_square, friendly_side) {
                break;
            }

            // target square is not empty, therefore is enemy piece => stop search after adding
            // piece to mask (only if the piece that is found can attack in this direction)
            if !target_empty {
                if board.piece_is_type(target_square, enemy_side, Pieces::QUEEN)
                    || (board.piece_is_type(target_square, enemy_side, Pieces::ROOK) && dir_idx < 4)
                    || (board.piece_is_type(target_square, enemy_side, Pieces::BISHOP)
                        && dir_idx >= 4)
                {
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

    board.king_attacker_mask =
        enemy_knight_attackers | enemy_pawn_attackers | enemy_sliding_piece_attackers;
}

pub fn calc_pinned_pieces(board: &mut Board, pos: u8) {
    let (friendly_side, enemy_side) = if board.is_white_to_play() {
        (Sides::WHITE, Sides::BLACK)
    } else {
        (Sides::BLACK, Sides::WHITE)
    };
    let en_passant = board.get_en_passant();

    board.pinned_pieces = BitBoard(0);
    board.pinned_pieces_move_mask = BitBoard(0);
    board.en_passant_pinned_piece = 65;

    for dir_idx in 0..8 {
        let dir = PRECOMPUTED_LOOKUPS.DIRECTION_OFFSETS[dir_idx as usize] as i8;

        let mut found_pinned_piece_idx = 65; // 65 => not found yet
        let mut found_pinned_piece_idx_en_passant = 65;
        let mut temp_pinned_move_mask = BitBoard(0);
        for n in 0..PRECOMPUTED_LOOKUPS.NUM_SQUARES_TO_EDGE[pos as usize][dir_idx] {
            let target_square = (pos as i8 + dir * (n + 1)) as u8;
            let target_piece_is_pawn =
                board.piece_is_type(target_square, Sides::WHITE, Pieces::PAWN)
                    || board.piece_is_type(target_square, Sides::BLACK, Pieces::PAWN);
            let target_piece_is_friendly = board.piece_color_on_tile(target_square, friendly_side);

            // if the direction is horizontal and the piece met is a pawn, check for en passant
            if (dir_idx == 2 || dir_idx == 3)
                && target_piece_is_pawn
                && found_pinned_piece_idx_en_passant == 65
            {
                // if there is a pawn, there are two scenarios: the first pawn is friendly and
                // the next one is the one to be en passanted, or the first one is en
                // passantable and the second one is the friendly pawn

                if !target_piece_is_friendly && en_passant == (target_square % 8) as u16 {
                    let next_piece_in_line_idx = (target_square as i8 + dir) as u8;
                    let next_piece_in_line_is_friendly_pawn =
                        board.piece_is_type(next_piece_in_line_idx, friendly_side, Pieces::PAWN);

                    if next_piece_in_line_is_friendly_pawn {
                        found_pinned_piece_idx_en_passant = (target_square as i8 + dir) as u8;
                        continue;
                    }
                }

                if target_piece_is_friendly
                    && en_passant == ((target_square as i8 + dir) % 8) as u16
                {
                    found_pinned_piece_idx_en_passant = target_square;
                    continue;
                }
            }

            let target_piece_is_empty = board.tile_is_empty(target_square);
            // target square is not empty and there is a friendly piece => if it is the first
            // friendly piece remember it, if it is the second: skip this direction
            if !target_piece_is_empty && target_piece_is_friendly {
                if found_pinned_piece_idx != 65 {
                    break;
                } else {
                    found_pinned_piece_idx = target_square;
                }
            } else if !target_piece_is_empty {
                // target square is not empty, therefore is enemy piece => stop search after adding
                // piece to mask (only if the piece that is found can attack in this direction)

                if board.piece_is_type(target_square, enemy_side, Pieces::QUEEN)
                    || (board.piece_is_type(target_square, enemy_side, Pieces::ROOK) && dir_idx < 4)
                    || (board.piece_is_type(target_square, enemy_side, Pieces::BISHOP)
                        && dir_idx >= 4)
                {
                    if found_pinned_piece_idx_en_passant != 65 {
                        board.en_passant_pinned_piece = found_pinned_piece_idx_en_passant;
                    } else if found_pinned_piece_idx != 65 {
                        board.pinned_pieces |= BitBoard(1 << found_pinned_piece_idx);
                        // if pinning piece is found, add the pinning piece to the move mask
                        temp_pinned_move_mask |= BitBoard(1 << target_square);
                        board.pinned_pieces_move_mask |= temp_pinned_move_mask;
                    }
                }
                break;
            }

            // if the square is empty, add it to the temp mask, only to add it to the pinned pieces
            // move mask, if a pinned piece is found later on
            temp_pinned_move_mask |= BitBoard(1 << target_square);
        }
    }
}
