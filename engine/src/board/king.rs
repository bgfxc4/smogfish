use super::bitboard::BitBoard;
use super::helper::{Color, Piece};
use super::precompute::{
    DIRECTION_OFFSETS, KING_CASTLE_CHECKS, KING_PAWN_ATTACKS, KNIGHT_ATTACKS, NUM_SQUARES_TO_EDGE,
};
use super::{Board, Move, Position};

pub fn get_all_moves_pseudolegal(board: &mut Board, pos: Position) {
    let friendly_side = board.current_player();

    for dir_idx in 0..8 {
        if NUM_SQUARES_TO_EDGE[pos.0 as usize][dir_idx] == 0 {
            continue;
        }
        let dir = DIRECTION_OFFSETS[dir_idx as usize] as i8;

        if (pos.0 as i8 + dir) >= 64 || (pos.0 as i8 + dir) < 0 {
            continue;
        }

        let target_square = (pos.0 as i8 + dir) as u8;

        // target square is not empty and there is a friendly piece => stop search in this direction
        if !board.tile_is_empty(Position(target_square))
            && board.piece_color_on_tile(Position(target_square), friendly_side)
        {
            continue;
        }

        // the target square is in check
        if board.check_mask & BitBoard(1 << target_square) != BitBoard(0) {
            continue;
        }

        board
            .move_list
            .push(Move::new(pos, Position(target_square)));
    }

    let total_castle_mask = board.white_total | board.black_total | board.check_mask;
    match friendly_side {
        Color::White => {
            if board.castle_white_short()
                && total_castle_mask & KING_CASTLE_CHECKS[Color::White as usize][0] == BitBoard(0)
            {
                board
                    .move_list
                    .push(Move::new_with_flags(pos, Position::new(0, 6), 3));
            }

            if board.castle_white_long()
                && total_castle_mask & KING_CASTLE_CHECKS[Color::White as usize][1] == BitBoard(0)
            {
                board
                    .move_list
                    .push(Move::new_with_flags(pos, Position::new(0, 2), 4));
            }
        }
        Color::Black => {
            if board.castle_black_short()
                && total_castle_mask & KING_CASTLE_CHECKS[Color::Black as usize][0] == BitBoard(0)
            {
                board
                    .move_list
                    .push(Move::new_with_flags(pos, Position::new(7, 6), 3));
            }

            if board.castle_black_long()
                && total_castle_mask & KING_CASTLE_CHECKS[Color::Black as usize][1] == BitBoard(0)
            {
                board
                    .move_list
                    .push(Move::new_with_flags(pos, Position::new(7, 2), 4));
            }
        }
    }
}

pub fn get_all_attacks(_board: &Board, pos: Position) -> BitBoard {
    let mut ret = BitBoard(0);

    for dir_idx in 0..8 {
        let dir = DIRECTION_OFFSETS[dir_idx as usize] as i8;

        if (pos.0 as i8 + dir) >= 64 || (pos.0 as i8 + dir) < 0 {
            continue;
        }

        let target_square = pos.0 as i8 + dir;
        ret |= BitBoard(1 << target_square);
    }
    ret
}

// the enemy king can be ignored here, he can not be an attacker of the own king
pub fn calc_king_attacker_masks(board: &mut Board, pos: Position) {
    let friendly_side = board.current_player();
    let enemy_side = !friendly_side;

    let enemy_knight_attackers =
        board.pieces[(enemy_side, Piece::Knight)] & KNIGHT_ATTACKS[pos.0 as usize];

    let enemy_pawn_attackers = board.pieces[(enemy_side, Piece::Pawn)]
        & KING_PAWN_ATTACKS[friendly_side as usize][pos.0 as usize];

    let mut enemy_sliding_piece_attackers = BitBoard(0);
    board.king_attacker_block_mask = BitBoard(0);
    for dir_idx in 0..8 {
        let dir = DIRECTION_OFFSETS[dir_idx as usize] as i8;

        let mut temp_block_mask = BitBoard(0);
        for n in 0..NUM_SQUARES_TO_EDGE[pos.0 as usize][dir_idx] {
            let target_square = Position((pos.0 as i8 + dir * (n + 1)) as u8);
            let target_empty = board.tile_is_empty(target_square);

            // target square is not empty and there is a friendly piece => stop search in this direction
            if !target_empty && board.piece_color_on_tile(target_square, friendly_side) {
                break;
            }

            // target square is not empty, therefore is enemy piece => stop search after adding
            // piece to mask (only if the piece that is found can attack in this direction)
            if !target_empty {
                if board.piece_is_type(target_square, enemy_side, Piece::Queen)
                    || (board.piece_is_type(target_square, enemy_side, Piece::Rook) && dir_idx < 4)
                    || (board.piece_is_type(target_square, enemy_side, Piece::Bishop)
                        && dir_idx >= 4)
                {
                    enemy_sliding_piece_attackers += target_square;
                    // if there is a sliding piece attacking the king, add all squares from the
                    // king to the sliding piece to the block mask
                    board.king_attacker_block_mask |= temp_block_mask;
                }
                break;
            }

            // if there is no piece on the square, add the square to the temp. mask
            temp_block_mask += target_square;
        }
    }

    board.king_attacker_mask =
        enemy_knight_attackers | enemy_pawn_attackers | enemy_sliding_piece_attackers;
}

pub fn calc_pinned_pieces(board: &mut Board, pos: Position) {
    let friendly_side = board.current_player();
    let enemy_side = !friendly_side;
    let en_passant = board.get_en_passant();

    board.pinned_pieces = BitBoard(0);
    board.pinned_pieces_move_mask = BitBoard(0);
    board.en_passant_pinned_piece = 65;

    for dir_idx in 0..8 {
        let dir = DIRECTION_OFFSETS[dir_idx as usize] as i8;

        let mut found_pinned_piece_idx = 65; // 65 => not found yet
        let mut found_pinned_piece_idx_en_passant = 65;
        let mut temp_pinned_move_mask = BitBoard(0);
        for n in 0..NUM_SQUARES_TO_EDGE[pos.0 as usize][dir_idx] {
            let target_square = Position((pos.0 as i8 + dir * (n + 1)) as u8);
            let target_piece_is_pawn =
                board.piece_is_type(target_square, Color::White, Piece::Pawn)
                    || board.piece_is_type(target_square, Color::Black, Piece::Pawn);
            let target_piece_is_friendly = board.piece_color_on_tile(target_square, friendly_side);

            // if the direction is horizontal and the piece met is a pawn, check for en passant
            if (dir_idx == 2 || dir_idx == 3)
                && target_piece_is_pawn
                && found_pinned_piece_idx_en_passant == 65
            {
                // if there is a pawn, there are two scenarios: the first pawn is friendly and
                // the next one is the one to be en passanted, or the first one is en
                // passantable and the second one is the friendly pawn

                if !target_piece_is_friendly && en_passant == target_square.file() as u16 {
                    let next_piece_in_line_idx = (target_square.0 as i8 + dir) as u8;
                    let next_piece_in_line_is_friendly_pawn = board.piece_is_type(
                        Position(next_piece_in_line_idx),
                        friendly_side,
                        Piece::Pawn,
                    );

                    if next_piece_in_line_is_friendly_pawn {
                        found_pinned_piece_idx_en_passant = (target_square.0 as i8 + dir) as u8;
                        continue;
                    }
                }

                if target_piece_is_friendly
                    && en_passant == ((target_square.0 as i8 + dir) % 8) as u16
                {
                    found_pinned_piece_idx_en_passant = target_square.0;
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
                    found_pinned_piece_idx = target_square.0;
                }
            } else if !target_piece_is_empty {
                // target square is not empty, therefore is enemy piece => stop search after adding
                // piece to mask (only if the piece that is found can attack in this direction)

                if board.piece_is_type(target_square, enemy_side, Piece::Queen)
                    || (board.piece_is_type(target_square, enemy_side, Piece::Rook) && dir_idx < 4)
                    || (board.piece_is_type(target_square, enemy_side, Piece::Bishop)
                        && dir_idx >= 4)
                {
                    if found_pinned_piece_idx_en_passant != 65 {
                        board.en_passant_pinned_piece = found_pinned_piece_idx_en_passant;
                    } else if found_pinned_piece_idx != 65 {
                        board.pinned_pieces += Position(found_pinned_piece_idx);
                        // if pinning piece is found, add the pinning piece to the move mask
                        temp_pinned_move_mask += target_square;
                        board.pinned_pieces_move_mask |= temp_pinned_move_mask;
                    }
                }
                break;
            }

            // if the square is empty, add it to the temp mask, only to add it to the pinned pieces
            // move mask, if a pinned piece is found later on
            temp_pinned_move_mask += target_square;
        }
    }
}