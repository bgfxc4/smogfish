use super::bitboard::BitBoard;
use super::helper::Pieces;
use super::helper::Sides;
use super::{Position, Board, Move};

pub fn get_all_moves_pseudolegal(board: &Board, pos: &Position, moves: &mut Vec<Move>) {
    // when the king is in double-check, the king has to move
    if board.king_attacker_count > 1 {
        return
    }

    let attacker_and_block_mask = if board.king_attacker_count == 0 { BitBoard(0) } else { board.king_attacker_block_mask | board.king_attacker_mask };

    let white_to_play = board.is_white_to_play();
    let modi: i8 = if white_to_play { 1 } else { -1 };
    let en_passant = board.get_en_passant();
    let is_pinned = board.pinned_pieces & BitBoard(1 << (pos.row*8+pos.col)) != BitBoard(0);
    
    // move one forward
    let p = Position::new(pos.col, ((pos.row as i8)+modi) as u8);
    if (board.king_attacker_count != 1 || attacker_and_block_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0)) &&
        (!is_pinned || board.pinned_pieces_move_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0)) {

        if board.get(&p).0 == Pieces::EMPTY {
            moves.push(Move::new(pos, &p));
        }
    }

    // move two forward
    let p = Position::new(pos.col, ((pos.row as i8)+modi*2) as u8);
    if board.king_attacker_count != 1 || attacker_and_block_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0) &&
        (!is_pinned || board.pinned_pieces_move_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0)) {

        let is_in_start_pos = (modi == 1 && pos.row == 1) || (modi == -1 && pos.row == 6);
        if is_in_start_pos && board.get(&p).0 == Pieces::EMPTY &&
            board.get(&Position::new(pos.col, ((pos.row as i8)+modi) as u8)).0 == Pieces::EMPTY {
            moves.push(Move::new_with_flags(pos, &p, 2));
        }
    }

    // take right
    if pos.col != 7 {
        let p = Position::new(pos.col+1, ((pos.row as i8)+modi) as u8);
        if board.king_attacker_count != 1 || attacker_and_block_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0) &&
            (!is_pinned || board.pinned_pieces_move_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0)) {

            let piece = board.get(&p);

            if ((piece.1 == Sides::WHITE) != white_to_play) && piece.0 != Pieces::EMPTY {
                moves.push(Move::new(pos, &p));
            } else if (white_to_play && (p.row == 5) && (en_passant == p.col as u16)) || // en passant
                      (!white_to_play && (p.row == 2) && (en_passant == p.col as u16)) {
                if board.en_passant_pinned_piece == 65 { // there can only be one en passant on the
                                                         // board => if one en passant piece is
                                                         // pinned, it is this one
                    moves.push(Move::new_with_flags(pos, &p, 1));
                }
            }
        }
    }

    // take left
    if pos.col != 0 {
        let p = Position::new(pos.col-1, ((pos.row as i8)+modi) as u8);
        if board.king_attacker_count != 1 || attacker_and_block_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0) &&
            (!is_pinned || board.pinned_pieces_move_mask & BitBoard(1 << (p.col+p.row*8)) != BitBoard(0)) {

            let piece = board.get(&p);
            if ((piece.1 == Sides::WHITE) != white_to_play) && piece.0 != Pieces::EMPTY {
                moves.push(Move::new(pos, &p));
            } else if (white_to_play && p.row == 5 && en_passant == p.col as u16) || // en passant
                      (!white_to_play && p.row == 2 && en_passant == p.col as u16) {
                if board.en_passant_pinned_piece == 65 {
                    moves.push(Move::new_with_flags(pos, &p, 1));
                }
            }
        }
    }
}

pub fn get_all_attacks(board: &Board, pos: &Position) -> BitBoard {
    let mut ret = BitBoard(0);
    let white_to_play = board.is_white_to_play();
    let modi: i8 = if white_to_play { -1 } else { 1 };
    
    // take right
    if pos.col != 7 {
        // check for piece on target square not necessary here
        let p = Position::new(pos.col+1, ((pos.row as i8)+modi) as u8);
        ret |= BitBoard(1 << (p.row*8+p.col));
    }

    // take left
    if pos.col != 0 {
        let p = Position::new(pos.col-1, ((pos.row as i8)+modi) as u8);
        ret |= BitBoard(1 << (p.row*8+p.col));
    }
    ret
}
