use super::helper::Pieces;
use super::helper::Sides;
use super::{Position, Board, Move};

pub fn get_all_moves_pseudolegal(board: &Board, pos: &Position, moves: &mut Vec<Move>) {
    let white_to_play = board.is_white_to_play();
    let modi: i8 = if white_to_play { 1 } else { -1 };
    let en_passant = board.get_en_passant();
    
    // move one forward
    let p = Position::new(pos.col, ((pos.row as i8)+modi) as u8);
    if board.get(&p).0 == Pieces::EMPTY {
        moves.push(Move::new(pos, &p));
    }

    // move two forward
    let p = Position::new(pos.col, ((pos.row as i8)+modi*2) as u8);
    let is_in_start_pos = (modi == 1 && pos.row == 1) || (modi == -1 && pos.row == 6);
    if is_in_start_pos && board.get(&p).0 == Pieces::EMPTY {
        moves.push(Move::new_with_flags(pos, &p, 2));
    }

    // take right
    if pos.col != 7 {
        let p = Position::new(pos.col+1, ((pos.row as i8)+modi) as u8);
        let piece = board.get(&p);
        if ((piece.1 == Sides::WHITE) != white_to_play) && piece.0 != Pieces::EMPTY {
            moves.push(Move::new(pos, &p));
        } else if (piece.1 == Sides::WHITE && p.row == 4 && en_passant == p.col) || // en passant
                  (piece.1 == Sides::BLACK && p.row == 3 && en_passant == p.col) {
            moves.push(Move::new_with_flags(pos, &p, 1));
        }
    }

    // take left
    if pos.col != 0 {
        let p = Position::new(pos.col-1, ((pos.row as i8)+modi) as u8);
        let piece = board.get(&p);
        if ((piece.1 == Sides::WHITE) != white_to_play) && piece.0 != Pieces::EMPTY {
            moves.push(Move::new(pos, &p));
        } else if (piece.1 == Sides::WHITE && p.row == 4 && en_passant == p.col) || // en passant
                  (piece.1 == Sides::BLACK && p.row == 3 && en_passant == p.col) {
            moves.push(Move::new_with_flags(pos, &p, 1));
        }
    }
}
