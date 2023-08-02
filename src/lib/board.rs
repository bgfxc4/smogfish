pub mod helper;
pub mod bitboard;
pub mod pawn;
pub mod sliding_pieces;
pub mod knight;
pub mod king;
pub mod precompute;
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
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }
}

pub struct Move {
    pub from: Position,
    pub to: Position,
    pub flag: u8, // 1 -> is_en_passant, 2 -> triggers_en_passant, 3 -> castle_short, 4 -> castle_long
}

impl Move {
    pub fn new(from: &Position, to: &Position) -> Self {
        Move{from: Position::new(from.col, from.row), to: Position::new(to.col, to.row), flag: 0}
    }

    pub fn new_with_flags(from: &Position, to: &Position, flag: u8) -> Self {
        Move{from: Position::new(from.col, from.row), to: Position::new(to.col, to.row), flag}
    }
}


pub struct Board {
    pieces: [[BitBoard; 6]; 2],
    white_total: BitBoard,
    black_total: BitBoard,
    check_mask: BitBoard, // marks every square, where a piece of the player, who just made a move
                          // is attacking. This bitboard ignores the king of the player whos turn
                          // it is
    king_attacker_count: u8,
    king_attacker_mask: BitBoard,
    king_attacker_block_mask: BitBoard,
    flags: u16, // side_to_play,
                // white_castle_short, white_castle_long, black_castle_short, black_castle_long
                // (all 1 bit)
                // en_passant_pos (4 bit number, 0-7, side is clear by active color, 13 (111b) to
                // signal no en passant)
    half_moves: u8, // for fifty move rule
    full_moves: u16,
}

impl Board {
    pub fn new(fen: &str) ->Self {
        lazy_static::initialize(&precompute::PRECOMPUTED_LOOKUPS);
        let mut b: Self = Board { pieces: [[BitBoard(0); 6]; 2],
            white_total: BitBoard(0),
            black_total: BitBoard(0),
            check_mask: BitBoard(0),
            king_attacker_count: 0,
            king_attacker_mask: BitBoard(0),
            king_attacker_block_mask: BitBoard(0),
            flags: 0,
            half_moves: 0,
            full_moves: 0};
        helper::load_board_from_fen(&mut b, fen).unwrap();
        return b;
    }

    fn set(&mut self, pos: &Position, piece: u8, color: u8) {
        self.pieces[color as usize][piece as usize] |= BitBoard(1 << pos.col+pos.row*8);
    }

    pub fn get(&self, pos: &Position) -> (u8, u8) {
        self.get_by_idx(pos.col+pos.row*8)
    }

    pub fn get_by_idx(&self, idx: u8) -> (u8, u8) { // dont use in engine, only for showing
                                                         // the board, not really efficient
        for p in 0..6 {
            for s in 0..2 {
                if self.pieces[s][p] & BitBoard(1 << idx) != BitBoard(0) {
                    return (p as u8, s as u8);
                }
            }
        }
        (Pieces::EMPTY, Sides::WHITE)
    }

    pub fn clear_bit(&mut self, pos: &Position, piece: u8, color: u8) {
        self.pieces[color as usize][piece as usize] &= BitBoard(!(1 << pos.col+pos.row*8));
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

    fn set_en_passant(&mut self, pos: u16) {
        self.flags &= !(13 << 5); // set the four bits to 0
        self.flags |= pos << 5; // set the pos to the four bits
    }

    fn get_en_passant(&self) -> u16 {
        let ret = self.flags >> 5; // bring four bits to the front
        return ret & (13); // set everyting else to 0
    }

    pub fn get_all_possible_moves(&self, pos: &Position) -> Vec<Move> {
        let mut pseudolegal_moves: Vec<Move> = vec![];
        let p = self.get(&pos);
        match p.0 {
            Pieces::PAWN => pawn::get_all_moves_pseudolegal(self, pos, &mut pseudolegal_moves),
            Pieces::KNIGHT => knight::get_all_moves(self, pos, &mut pseudolegal_moves),
            Pieces::BISHOP => sliding_pieces::get_all_moves_bishop_pseudolegal(self, pos, &mut pseudolegal_moves),
            Pieces::ROOK => sliding_pieces::get_all_moves_rook_pseudolegal(self, pos, &mut pseudolegal_moves),
            Pieces::QUEEN => sliding_pieces::get_all_moves_queen_pseudolegal(self, pos, &mut pseudolegal_moves),
            Pieces::KING => king::get_all_moves_pseudolegal(self, pos, &mut pseudolegal_moves),
            _ => (),
        };

        // TODO: filter out illegal moves

        return pseudolegal_moves;
    }

    fn generate_total_bitboard(&mut self, color: u8) {
        if color == Sides::WHITE {
            self.white_total = self.pieces[Sides::WHITE as usize][Pieces::PAWN as usize] | self.pieces[Sides::WHITE as usize][Pieces::KNIGHT as usize] | self.pieces[Sides::WHITE as usize][Pieces::BISHOP as usize] | self.pieces[Sides::WHITE as usize][Pieces::ROOK as usize] | self.pieces[Sides::WHITE as usize][Pieces::QUEEN as usize] | self.pieces[Sides::WHITE as usize][Pieces::KING as usize];
        } else {
            self.black_total = self.pieces[Sides::BLACK as usize][Pieces::PAWN as usize] | self.pieces[Sides::BLACK as usize][Pieces::KNIGHT as usize] | self.pieces[Sides::BLACK as usize][Pieces::BISHOP as usize] | self.pieces[Sides::BLACK as usize][Pieces::ROOK as usize] | self.pieces[Sides::BLACK as usize][Pieces::QUEEN as usize] | self.pieces[Sides::BLACK as usize][Pieces::KING as usize];
        }
    }

    fn generate_check_mask(&mut self, color: u8) {
        let mut king_pos: u8 = 0;
        self.check_mask = BitBoard(0);
        for idx in 0..64 {
            let pos = &Position { col: idx%8, row: idx/8 };
            let piece = self.get_by_idx(idx);
            if piece.1 != color {
                if piece.0 == Pieces::KING {
                    king_pos = idx;
                }
                continue;
            }
            self.check_mask |= match piece.0 {
                Pieces::PAWN => pawn::get_all_attacks(self, pos),
                Pieces::KNIGHT => knight::get_all_attacks(self, pos),
                Pieces::BISHOP => sliding_pieces::get_all_attacks_bishop(self, pos, color),
                Pieces::ROOK => sliding_pieces::get_all_attacks_rook(self, pos, color),
                Pieces::QUEEN => sliding_pieces::get_all_attacks_queen(self, pos, color),
                Pieces::KING => king::get_all_attacks(self, pos),
                _ => BitBoard(0),
            };
        }
        self.check_mask.print();
        if (self.check_mask & BitBoard(1 << king_pos)) != BitBoard(0) {
            king::calc_king_attacker_masks(self, king_pos);
            self.king_attacker_count = self.king_attacker_mask.count_set_bits();

            println!("King is in check: {} attackers", self.king_attacker_count);
            self.king_attacker_mask.print();
            self.king_attacker_block_mask.print();
        } else {
            self.king_attacker_count = 0;
            self.king_attacker_block_mask = BitBoard(0);
            self.king_attacker_mask = BitBoard(0);
        }
    }

    #[inline]
    pub fn is_white_to_play(&self) -> bool {
        self.flags & (1) == Sides::WHITE as u16
    }

    pub fn make_move(&mut self, mov: &Move) {
        let p = self.get(&mov.from);
        let target_piece = self.get(&mov.to);
        let move_is_capture = target_piece.0 != Pieces::EMPTY;
        let (side_to_play, oponent_side) = if self.is_white_to_play() { (Sides::WHITE, Sides::BLACK) } else { (Sides::BLACK, Sides::WHITE) };


        self.clear_bit(&mov.from, p.0, side_to_play);
        if move_is_capture {
            self.clear_bit(&mov.to, target_piece.0, target_piece.1);
        }
        self.set(&mov.to, p.0, p.1);

        if mov.flag == 1 { // move is en passant
            // TODO: better and more efficient board.clear_field to avoid use of board.get here
            self.clear_bit(&Position::new(mov.to.col, (mov.from.row as i8) as u8), self.get(&mov.to).0, oponent_side);
        } else if mov.flag == 2 { // move triggers en passant
            self.set_en_passant(mov.to.col as u16);
        }

        if (p.0 != Pieces::PAWN) && !move_is_capture {
            self.half_moves += 1;
        } else {
            self.half_moves = 1;
        }

        let mut next_color_to_move = Sides::BLACK;
        if !self.is_white_to_play() {
            self.full_moves += 1;
            next_color_to_move = Sides::WHITE;
        }
        self.set_color_to_move(next_color_to_move);
        if mov.flag != 2 { // if en passant didnt just get triggered, reset it
            self.set_en_passant(13);
        }

        self.generate_total_bitboard(side_to_play);
        self.generate_check_mask(side_to_play);
    }
}
