pub mod helper;
pub mod bitboard;
pub mod pawn;
pub mod sliding_pieces;
pub mod knight;
pub mod king;
pub mod precompute;
use helper::{Sides, Pieces};
use bitboard::BitBoard;

use self::{precompute::PRECOMPUTED_LOOKUPS, helper::GameState};

pub type Position = u8;

#[derive(Clone)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub flag: u8, // 1 -> is_en_passant, 2 -> triggers_en_passant, 3 -> castle_short, 4 -> castle_long, 5 -> promote queen, 6 -> promote rook, 7 -> promote bishop, 8 -> promote knight
}

impl Move {
    pub fn new(from: Position, to: Position) -> Self {
        Move{from, to, flag: 0}
    }

    pub fn new_with_flags(from: Position, to: Position, flag: u8) -> Self {
        Move{from, to, flag}
    }
}

#[derive(Clone)]
pub struct Board {
    pub game_state: u8,
    pieces: [[BitBoard; 6]; 2],
    white_total: BitBoard,
    black_total: BitBoard,
    check_mask: BitBoard, // marks every square, where a piece of the player, who just made a move
                          // is attacking. This bitboard ignores the king of the player whos turn
                          // it is
    king_attacker_count: u8,
    king_attacker_mask: BitBoard,
    king_attacker_block_mask: BitBoard,
    pinned_pieces: BitBoard,
    pinned_pieces_move_mask: BitBoard,
    en_passant_pinned_piece: u8, // stores, if a pawn can not take en passant. This can only be one
                                 // pawn at once and the tile index of it is stored here.
                                 // 65 -> empty
    flags: u16, // side_to_play,
                // white_castle_short, white_castle_long, black_castle_short, black_castle_long
                // (all 1 bit)
                // en_passant_pos (4 bit number, 0-7, side is clear by active color, 15 (1111b) to
                // signal no en passant)
    half_moves: u8, // for fifty move rule
    full_moves: u16,
    zobrist_history: [u64; 101], // for threefold repetition, indexed by halfmoves 
    pub move_list: Vec<Move>,
}

impl Board {
    pub fn new(fen: &str) ->Self {
        lazy_static::initialize(&precompute::PRECOMPUTED_LOOKUPS);
        let mut b: Self = Board { pieces: [[BitBoard(0); 6]; 2],
            game_state: GameState::PLAYING,
            white_total: BitBoard(0),
            black_total: BitBoard(0),
            check_mask: BitBoard(0),
            king_attacker_count: 0,
            king_attacker_mask: BitBoard(0),
            king_attacker_block_mask: BitBoard(0),
            pinned_pieces: BitBoard(0),
            pinned_pieces_move_mask: BitBoard(0),
            en_passant_pinned_piece: 65,
            flags: 0,
            half_moves: 0,
            full_moves: 0,
            zobrist_history: [0; 101],
            move_list: vec![],
        };
        helper::load_board_from_fen(&mut b, fen).unwrap();
        return b;
    }

    fn set(&mut self, pos: Position, piece: u8, color: u8) {
        self.pieces[color as usize][piece as usize] |= BitBoard(1 << pos);
    }

    #[inline]
    pub fn tile_is_empty(&self, idx: Position) -> bool {
        (self.white_total | self.black_total) & BitBoard(1 << idx) == BitBoard(0)
    }

    #[inline]
    pub fn piece_color_on_tile(&self, idx: Position, color: u8) -> bool {
        if color == Sides::WHITE { self.white_total & BitBoard(1 << idx) != BitBoard(0) } else { self.black_total & BitBoard(1 << idx) != BitBoard(0) }
    }

    #[inline]
    pub fn piece_is_sliding(&self, idx: Position, color: u8) -> bool {
        (self.pieces[color as usize][Pieces::QUEEN as usize] | self.pieces[color as usize][Pieces::ROOK as usize] | self.pieces[color as usize][Pieces::BISHOP as usize]) & BitBoard(1 << idx) != BitBoard(0)
    }

    #[inline]
    pub fn piece_is_type(&self, idx: Position, color: u8, piece: u8) -> bool {
        self.pieces[color as usize][piece as usize] & BitBoard(1 << idx) != BitBoard(0)
    }

    pub fn get_by_idx(&self, idx: Position) -> (u8, u8) { // dont use in engine, only for showing
                                                         // the board, not really efficient
        if self.white_total & BitBoard(1 << idx) != BitBoard(0) {
            for p in 0..6 {
                if self.pieces[Sides::WHITE as usize][p] & BitBoard(1 << idx) != BitBoard(0) {
                    return (p as u8, Sides::WHITE);
                }
            }
        }
        if self.black_total & BitBoard(1 << idx) != BitBoard(0) {
            for p in 0..6 {
                if self.pieces[Sides::BLACK as usize][p] & BitBoard(1 << idx) != BitBoard(0) {
                    return (p as u8, Sides::BLACK);
                }
            }
        }
        (Pieces::EMPTY, Sides::WHITE)
    }

    pub fn clear_bit(&mut self, pos: Position, piece: u8, color: u8) {
        self.pieces[color as usize][piece as usize] &= BitBoard(!(1 << pos));
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

    #[inline]
    fn castle_white_short(&self) -> bool {
        self.flags & 1 << 1 != 0
    }

    #[inline]
    fn castle_white_long(&self) -> bool {
        self.flags & 1 << 2 != 0
    }

    #[inline]
    fn castle_black_short(&self) -> bool {
        self.flags & 1 << 3 != 0
    }

    #[inline]
    fn castle_black_long(&self) -> bool {
        self.flags & 1 << 4 != 0
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
        self.flags &= !(15 << 5); // set the four bits to 0
        self.flags |= pos << 5; // set the pos to the four bits
    }

    fn get_en_passant(&self) -> u16 {
        let ret = self.flags >> 5; // bring four bits to the front
        return ret & (15); // set everyting else to 0
    }

    fn compute_zobrist_hash(&self) -> u64 {
        let mut hash_value = 0;
        for i in 0..64 {
            for p in 0..Pieces::EMPTY {
                if self.pieces[Sides::WHITE as usize][p as usize] & BitBoard(1 << i) != BitBoard(0) {
                    hash_value ^= PRECOMPUTED_LOOKUPS.ZOBRIST_HASH_TABLE[i][p as usize];
                } else if self.pieces[Sides::BLACK as usize][p as usize] & BitBoard(1 << i) != BitBoard(0) {
                    hash_value ^= PRECOMPUTED_LOOKUPS.ZOBRIST_HASH_TABLE[i][p as usize + 6];
                }
            }
        }
        if !self.is_white_to_play() {
            hash_value ^= PRECOMPUTED_LOOKUPS.ZOBRIST_SPECIAL_KEYS[0];
        }
        if self.castle_white_short() {
            hash_value ^= PRECOMPUTED_LOOKUPS.ZOBRIST_SPECIAL_KEYS[1];
        }
        if self.castle_white_long() {
            hash_value ^= PRECOMPUTED_LOOKUPS.ZOBRIST_SPECIAL_KEYS[2];
        }
        if self.castle_black_short() {
            hash_value ^= PRECOMPUTED_LOOKUPS.ZOBRIST_SPECIAL_KEYS[3];
        }
        if self.castle_black_long() {
            hash_value ^= PRECOMPUTED_LOOKUPS.ZOBRIST_SPECIAL_KEYS[4];
        }

        hash_value
    }

    pub fn generate_move_list(&mut self) {
        self.move_list.clear();
        let mut move_list: Vec<Move> = vec![];
        let side_to_play = if self.is_white_to_play() { Sides::WHITE } else { Sides::BLACK };
        for p in 0..6 {
            for i in 0..64 {
                if self.pieces[side_to_play as usize][p] & BitBoard(1 << i) != BitBoard(0) {
                    match p as u8 {
                        Pieces::PAWN => pawn::get_all_moves_pseudolegal(self, i, &mut move_list),
                        Pieces::KNIGHT => knight::get_all_moves(self, i, &mut move_list),
                        Pieces::BISHOP => sliding_pieces::get_all_moves_bishop_pseudolegal(self, i, &mut move_list),
                        Pieces::ROOK => sliding_pieces::get_all_moves_rook_pseudolegal(self, i, &mut move_list),
                        Pieces::QUEEN => sliding_pieces::get_all_moves_queen_pseudolegal(self, i, &mut move_list),
                        Pieces::KING => king::get_all_moves_pseudolegal(self, i, &mut move_list),
                        _ => (),
                    };
                }
            }
        }
        self.move_list = move_list;
    }

    fn generate_total_bitboard(&mut self, color: u8) {
        if color == Sides::WHITE {
            self.white_total = self.pieces[Sides::WHITE as usize][Pieces::PAWN as usize] | self.pieces[Sides::WHITE as usize][Pieces::KNIGHT as usize] | self.pieces[Sides::WHITE as usize][Pieces::BISHOP as usize] | self.pieces[Sides::WHITE as usize][Pieces::ROOK as usize] | self.pieces[Sides::WHITE as usize][Pieces::QUEEN as usize] | self.pieces[Sides::WHITE as usize][Pieces::KING as usize];
        } else {
            self.black_total = self.pieces[Sides::BLACK as usize][Pieces::PAWN as usize] | self.pieces[Sides::BLACK as usize][Pieces::KNIGHT as usize] | self.pieces[Sides::BLACK as usize][Pieces::BISHOP as usize] | self.pieces[Sides::BLACK as usize][Pieces::ROOK as usize] | self.pieces[Sides::BLACK as usize][Pieces::QUEEN as usize] | self.pieces[Sides::BLACK as usize][Pieces::KING as usize];
        }
    }

    fn generate_check_mask(&mut self, color: u8) { // color => enemy color
        let enemy_side = if color == Sides::WHITE { Sides::BLACK } else { Sides::WHITE };
        let mut king_pos: u8 = 0;
        self.check_mask = BitBoard(0);
        for idx in 0..64 {
            if !self.piece_color_on_tile(idx, color) {
                if self.piece_is_type(idx, enemy_side, Pieces::KING) {
                    king_pos = idx;
                }
                continue;
            }

            if self.piece_is_type(idx, color, Pieces::PAWN) {
                self.check_mask |= pawn::get_all_attacks(self, idx);
            } else if self.piece_is_type(idx, color, Pieces::KNIGHT) {
                self.check_mask |= knight::get_all_attacks(self, idx);
            } else if self.piece_is_type(idx, color, Pieces::BISHOP) {
                self.check_mask |= sliding_pieces::get_all_attacks_bishop(self, idx, color);
            } else if self.piece_is_type(idx, color, Pieces::ROOK) {
                self.check_mask |= sliding_pieces::get_all_attacks_rook(self, idx, color);
            } else if self.piece_is_type(idx, color, Pieces::QUEEN) {
                self.check_mask |= sliding_pieces::get_all_attacks_queen(self, idx, color);
            } else if self.piece_is_type(idx, color, Pieces::KING) {
                self.check_mask |= king::get_all_attacks(self, idx);
            }

        }
        if (self.check_mask & BitBoard(1 << king_pos)) != BitBoard(0) {
            king::calc_king_attacker_masks(self, king_pos);
            self.king_attacker_count = self.king_attacker_mask.count_set_bits();
        } else {
            self.king_attacker_count = 0;
            self.king_attacker_block_mask = BitBoard(0);
            self.king_attacker_mask = BitBoard(0);
        }

        king::calc_pinned_pieces(self, king_pos);
    }

    #[inline]
    pub fn is_white_to_play(&self) -> bool {
        self.flags & (1) == Sides::WHITE as u16
    }

    pub fn make_move(&mut self, mov: &Move) {
        let p = self.get_by_idx(mov.from);
        let target_piece = self.get_by_idx(mov.to);
        let move_is_capture = target_piece.0 != Pieces::EMPTY;
        let (side_to_play, oponent_side) = if self.is_white_to_play() { (Sides::WHITE, Sides::BLACK) } else { (Sides::BLACK, Sides::WHITE) };


        self.clear_bit(mov.from, p.0, side_to_play);
        if move_is_capture {
            self.clear_bit(mov.to, target_piece.0, target_piece.1);
        }

        self.set(mov.to, p.0, p.1);

        match mov.flag {
            3 => {
                if side_to_play == Sides::WHITE {
                    self.clear_bit(7+0*8, Pieces::ROOK, Sides::WHITE);
                    self.set(5+0*8, Pieces::ROOK, Sides::WHITE);
                    self.remove_castling_right(Sides::WHITE, false);
                    self.remove_castling_right(Sides::WHITE, true);
                } else {
                    self.clear_bit(7+7*8, Pieces::ROOK, Sides::BLACK);
                    self.set(5+7*8, Pieces::ROOK, Sides::BLACK);
                    self.remove_castling_right(Sides::BLACK, false);
                    self.remove_castling_right(Sides::BLACK, true);
                }
            },
            4 => {
                if side_to_play == Sides::WHITE {
                    self.clear_bit(0+0*8, Pieces::ROOK, Sides::WHITE);
                    self.set(3+0*8, Pieces::ROOK, Sides::WHITE);
                    self.remove_castling_right(Sides::WHITE, false);
                    self.remove_castling_right(Sides::WHITE, true);
                } else {
                    self.clear_bit(0+7*8, Pieces::ROOK, Sides::BLACK);
                    self.set(3+7*8, Pieces::ROOK, Sides::BLACK);
                    self.remove_castling_right(Sides::BLACK, false);
                    self.remove_castling_right(Sides::BLACK, true);
                }
            },
            5 => self.set(mov.to, Pieces::QUEEN, p.1), // if promotion, set new piece on target instead of old one
            6 => self.set(mov.to, Pieces::ROOK, p.1),
            7 => self.set(mov.to, Pieces::BISHOP, p.1),
            8 => self.set(mov.to, Pieces::KNIGHT, p.1),
            _ => (),
        } 

        if mov.flag == 1 { // move is en passant
            // TODO: better and more efficient board.clear_field to avoid use of board.get here
            self.clear_bit((mov.to%8)+(mov.from/8)*8, Pieces::PAWN, oponent_side);
        } else if mov.flag == 2 { // move triggers en passant
            self.set_en_passant((mov.to%8) as u16);
        }

        if p.0 == Pieces::KING {
            if side_to_play == Sides::WHITE && (self.castle_white_short() || self.castle_white_long()) {
                self.remove_castling_right(Sides::WHITE, false);
                self.remove_castling_right(Sides::WHITE, true);
            } else if side_to_play == Sides::BLACK && (self.castle_black_short() || self.castle_black_long()) {
                self.remove_castling_right(Sides::BLACK, false);
                self.remove_castling_right(Sides::BLACK, true);
            }
        } else if p.0 == Pieces::ROOK {
            if side_to_play == Sides::WHITE {
                if mov.from == 0+0*8{
                    self.remove_castling_right(Sides::WHITE, true);
                } else if mov.from== 7+0*8 {
                    self.remove_castling_right(Sides::WHITE, false);
                }
            } else {
                if mov.from == 0+7*8 {
                    self.remove_castling_right(Sides::BLACK, true);
                } else if mov.from == 7+7*8 {
                    self.remove_castling_right(Sides::BLACK, false);
                }
            }
        }

        if (p.0 != Pieces::PAWN) && !move_is_capture {
            self.half_moves += 1;
            
            if self.half_moves >= 100 {
                self.game_state = GameState::DRAW;
            }

            let hash = self.compute_zobrist_hash();
            let mut repetition_count = 0;
            for i in 0..self.half_moves {
                if self.zobrist_history[i as usize] == hash {
                    repetition_count += 1;
                    if repetition_count == 2 {
                        self.game_state = GameState::DRAW;
                        break;
                    }
                }
            }
            self.zobrist_history[self.half_moves as usize] = hash;
        } else {
            self.half_moves = 0;
            self.zobrist_history = [0; 101];
            if mov.flag == 2 { // if the new position includes a possible en passant, dont add it
                               // to the zobrist history
                self.zobrist_history[0] = 0;
            } else {
                self.zobrist_history[0] = self.compute_zobrist_hash();
            }
        }

        let mut next_color_to_move = Sides::BLACK;
        if !self.is_white_to_play() {
            self.full_moves += 1;
            next_color_to_move = Sides::WHITE;
        }
        self.set_color_to_move(next_color_to_move);
        if mov.flag != 2 { // if en passant didnt just get triggered, reset it
            self.set_en_passant(15);
        }

        self.generate_total_bitboard(side_to_play);
        if move_is_capture {
            self.generate_total_bitboard(next_color_to_move);
        }
        self.generate_check_mask(side_to_play);

        self.generate_move_list();

        if self.move_list.len() == 0 {
            if self.king_attacker_count == 0 {
                self.game_state = GameState::DRAW;
            } else {
                self.game_state = match next_color_to_move {
                    Sides::WHITE => GameState::BLACK_WINS,
                    Sides::BLACK => GameState::WHITE_WINS,
                    _ => GameState::WHITE_WINS,
                }
            }
        }
    }
}
