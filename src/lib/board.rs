pub mod helper;
pub mod bitboard;
pub mod pawn;
pub mod sliding_pieces;
pub mod knight;
pub mod king;
use helper::{Sides, Pieces};
use bitboard::BitBoard;

use std::cmp::PartialEq;

pub struct Position {
    pub col: u8,
    pub row: u8,
}

impl Position {
    pub fn new(col: u8, row: u8) -> Self {
        Position{col, row}
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }
}

pub struct Move {
    pub from: Position,
    pub to: Position,
}

impl Move {
    pub fn new(from: Position, to: Position) -> Self {
        Move{from, to}
    }
}


pub struct Board {
    pieces: [[BitBoard; 6]; 2],
    white_total: BitBoard,
    black_total: BitBoard,
    flags: u8, // side_to_play,
                // white_castle_short, white_castle_long, black_castle_short, black_castle_long
                // (all 1 bit)
                // en_passant_pos (3 bit number, 0-7, side is clear by active color)
    half_moves: u8, // for fifty move rule
    full_moves: u16,
}

impl Board {
    pub fn new(fen: &str) ->Self {
        let mut b: Self = Board { pieces: [[BitBoard(0); 6]; 2], white_total: BitBoard(0), black_total: BitBoard(0), flags: 0, half_moves: 0, full_moves: 0};
        helper::load_board_from_fen(&mut b, fen).unwrap();
        return b;
    }

    fn set(&mut self, pos: &Position, piece: u8, color: u8) {
        self.pieces[color as usize][piece as usize] |= BitBoard(1 << pos.col+pos.row*8);
    }

    pub fn get(&self, pos: &Position) -> (u8, u8) { // dont use in engine, only for showing
                                                         // the board, not really efficient
        for p in 0..6 {
            for s in 0..2 {
                if self.pieces[s][p] & BitBoard(1 << pos.col+pos.row*8) != BitBoard(0) {
                    return (p as u8, s as u8);
                }
            }
        }
        (Pieces::EMPTY, Sides::WHITE)
    }

    fn set_color_to_move(&mut self, color: u8) {
        if color == Sides::WHITE {
            self.flags &= !1;
        } else {
            self.flags |= 1;
        }
    }

    fn set_castling_right(&mut self, color: u8, is_long: bool) {
        if color == Sides::WHITE {
            if is_long {
                self.flags |= 1 << 2;
            } else {
                self.flags |= 1 << 1;
            }
        } else {
            if is_long {
                self.flags |= 1 << 4;
            } else {
                self.flags |= 1 << 3;
            }
        }
    }

    fn remove_castling_right(&mut self, color: u8, is_long: bool) {
        if color == Sides::WHITE {
            if is_long {
                self.flags &= !(1 << 2);
            } else {
                self.flags &= !(1 << 1);
            }
        } else {
            if is_long {
                self.flags &= !(1 << 4);
            } else {
                self.flags &= !(1 << 3);
            }
        }
    }

    fn set_en_passant(&mut self, pos: u8) {
        self.flags &= !(7 << 5); // set the three bits to 0
        self.flags |= pos << 5; // set the pos to the three bits
        println!("set passant to {}", pos)
    }

    pub fn get_all_possible_moves(&self, pos: &Position) -> Vec<Move> {
        let p = self.get(&pos);
        return match p.0 {
            Pieces::PAWN => pawn::get_all_moves(self, pos),
            Pieces::KNIGHT => knight::get_all_moves(self, pos),
            Pieces::BISHOP => sliding_pieces::get_all_moves_bishop(self, pos),
            Pieces::ROOK => sliding_pieces::get_all_moves_rook(self, pos),
            Pieces::QUEEN => sliding_pieces::get_all_moves_queen(self, pos),
            Pieces::KING => king::get_all_moves(self, pos),
            _ => vec![],
        }
    }
}
