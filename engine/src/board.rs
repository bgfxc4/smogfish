pub mod bitboard;
pub mod helper;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod precompute;
pub mod sliding_pieces;

use self::{
    helper::{Color, GameState, PieceBoards, Position},
    precompute::{ZOBRIST_HASH_TABLE, ZOBRIST_SPECIAL_KEYS},
};
use bitboard::BitBoard;
use helper::Piece;

#[derive(Clone, Debug)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    /// 1 -> is_en_passant
    /// 2 -> triggers_en_passant
    /// 3 -> castle_short
    /// 4 -> castle_long
    /// 5 -> promote queen
    /// 6 -> promote rook
    /// 7 -> promote bishop
    /// 8 -> promote knight
    pub flag: u8,
}

impl Move {
    pub fn new(from: Position, to: Position) -> Self {
        Move { from, to, flag: 0 }
    }

    pub fn new_with_flags(from: Position, to: Position, flag: u8) -> Self {
        Move { from, to, flag }
    }
}

#[derive(Clone)]
pub struct Board {
    pub game_state: GameState,
    pieces: PieceBoards,
    white_total: BitBoard,
    black_total: BitBoard,
    /// marks every square, where a piece of the player, who just made a move
    /// is attacking. This bitboard ignores the king of the player whos turn
    /// it is
    check_mask: BitBoard,
    king_attacker_count: u8,
    king_attacker_mask: BitBoard,
    king_attacker_block_mask: BitBoard,
    pinned_pieces: BitBoard,
    pinned_pieces_move_masks: [BitBoard; 64],
    /// stores, if a pawn can not take en passant. This can only be one
    /// pawn at once and the tile index of it is stored here.
    /// 65 -> empty
    en_passant_pinned_piece: u8,
    /// 1 -> side_to_play,
    /// 2 -> white_castle_short
    /// 3 -> white_castle_long
    /// 4 -> black_castle_short
    /// 5 -> black_castle_long
    /// 5..9 -> en_passant_pos (4 bit number, 0-7, side is clear by active color, 15 (1111b) to signal no en passant)
    flags: u16,
    /// for fifty move rule
    half_moves: u8,
    full_moves: u16,
    /// for threefold repetition, indexed by halfmoves
    zobrist_history: [u64; 101],
    pub move_list: Vec<Move>,
}

impl Board {
    pub fn new(fen: &str) -> Self {
        let mut b: Self = Board {
            pieces: Default::default(),
            game_state: GameState::Playing,
            white_total: BitBoard(0),
            black_total: BitBoard(0),
            check_mask: BitBoard(0),
            king_attacker_count: 0,
            king_attacker_mask: BitBoard(0),
            king_attacker_block_mask: BitBoard(0),
            pinned_pieces: BitBoard(0),
            pinned_pieces_move_masks: [BitBoard(0); 64],
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

    fn set(&mut self, pos: Position, piece: Piece, color: Color) {
        self.pieces[(color, piece)] += pos;
    }

    #[inline]
    pub fn tile_is_empty(&self, idx: Position) -> bool {
        !(self.white_total | self.black_total).has(idx)
    }

    #[inline]
    pub fn piece_color_on_tile(&self, idx: Position, color: Color) -> bool {
        if color == Color::White {
            self.white_total.has(idx)
        } else {
            self.black_total.has(idx)
        }
    }

    #[inline]
    pub fn piece_is_sliding(&self, idx: Position, color: Color) -> bool {
        (self.pieces[(color, Piece::Queen)]
            | self.pieces[(color, Piece::Rook)]
            | self.pieces[(color, Piece::Bishop)])
            .has(idx)
    }

    #[inline]
    pub fn piece_is_type(&self, idx: Position, color: Color, piece: Piece) -> bool {
        self.pieces[(color, piece)].has(idx)
    }

    /// dont use in engine, only for showing
    /// the board, not really efficient
    pub fn get_by_idx(&self, idx: Position) -> (Piece, Color) {
        if self.white_total.has(idx) {
            for p in Piece::ALL_NONEMPTY {
                if self.pieces[(Color::White, p)].has(idx) {
                    return (p, Color::White);
                }
            }
        }
        if self.black_total.has(idx) {
            for p in Piece::ALL_NONEMPTY {
                if self.pieces[(Color::Black, p)].has(idx) {
                    return (p, Color::Black);
                }
            }
        }
        (Piece::Empty, Color::White)
    }

    pub fn clear_bit(&mut self, pos: Position, piece: Piece, color: Color) {
        self.pieces[(color, piece)] -= pos;
    }

    fn set_color_to_move(&mut self, color: Color) {
        if color == Color::White {
            self.flags &= !1;
        } else {
            self.flags |= 1;
        }
    }

    fn set_castling_right(&mut self, color: Color, is_long: bool) {
        if color == Color::White {
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

    fn remove_castling_right(&mut self, color: Color, is_long: bool) {
        if color == Color::White {
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
        for p in Piece::ALL_NONEMPTY {
            for i in self.pieces[(Color::White, p)] {
                hash_value ^= ZOBRIST_HASH_TABLE[i.0 as usize][p as usize];
            }
            for i in self.pieces[(Color::Black, p)] {
                hash_value ^= ZOBRIST_HASH_TABLE[i.0 as usize][p as usize + 6];
            }
        }
        if self.current_player() == Color::Black {
            hash_value ^= ZOBRIST_SPECIAL_KEYS[0];
        }
        if self.castle_white_short() {
            hash_value ^= ZOBRIST_SPECIAL_KEYS[1];
        }
        if self.castle_white_long() {
            hash_value ^= ZOBRIST_SPECIAL_KEYS[2];
        }
        if self.castle_black_short() {
            hash_value ^= ZOBRIST_SPECIAL_KEYS[3];
        }
        if self.castle_black_long() {
            hash_value ^= ZOBRIST_SPECIAL_KEYS[4];
        }

        hash_value
    }

    pub fn generate_move_list(&mut self) {
        self.move_list.clear();
        let side_to_play = self.current_player();
        for p in Piece::ALL_NONEMPTY {
            for i in self.pieces[(side_to_play, p)] {
                match p {
                    Piece::Pawn => pawn::get_all_moves(self, i),
                    Piece::Knight => knight::get_all_moves(self, i),
                    Piece::Bishop => sliding_pieces::get_all_moves_bishop(self, i),
                    Piece::Rook => sliding_pieces::get_all_moves_rook(self, i),
                    Piece::Queen => sliding_pieces::get_all_moves_queen(self, i),
                    Piece::King => king::get_all_moves(self, i),
                    _ => (),
                };
            }
        }
    }

    fn generate_total_bitboard(&mut self, color: Color) {
        if color == Color::White {
            self.white_total = self.pieces[(Color::White, Piece::Pawn)]
                | self.pieces[(Color::White, Piece::Knight)]
                | self.pieces[(Color::White, Piece::Bishop)]
                | self.pieces[(Color::White, Piece::Rook)]
                | self.pieces[(Color::White, Piece::Queen)]
                | self.pieces[(Color::White, Piece::King)];
        } else {
            self.black_total = self.pieces[(Color::Black, Piece::Pawn)]
                | self.pieces[(Color::Black, Piece::Knight)]
                | self.pieces[(Color::Black, Piece::Bishop)]
                | self.pieces[(Color::Black, Piece::Rook)]
                | self.pieces[(Color::Black, Piece::Queen)]
                | self.pieces[(Color::Black, Piece::King)];
        }
    }

    fn generate_check_mask(&mut self, color: Color) {
        // color => enemy color
        let king_pos = self.pieces[(!color, Piece::King)]
            .into_iter()
            .next()
            .unwrap();
        self.check_mask = BitBoard(0);
        for i in self.pieces[(color, Piece::Knight)].into_iter() {
            self.check_mask |= knight::get_all_attacks(self, i);
        }
        for i in self.pieces[(color, Piece::King)].into_iter() {
            self.check_mask |= king::get_all_attacks(self, i);
        }

        self.check_mask |= pawn::get_all_attacks(self, self.pieces[(color, Piece::Pawn)]);
        self.check_mask |= sliding_pieces::get_all_attacks_rook(self, self.pieces[(color, Piece::Rook)] | self.pieces[(color, Piece::Queen)], color);
        self.check_mask |= sliding_pieces::get_all_attacks_bishop(self, self.pieces[(color, Piece::Bishop)] | self.pieces[(color, Piece::Queen)], color);

        if self.check_mask.has(king_pos) {
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
    pub fn current_player(&self) -> Color {
        match self.flags & (1) {
            0 => Color::White,
            1 => Color::Black,
            _ => unreachable!(),
        }
    }

    pub fn make_move(&mut self, mov: &Move) {
        let p = self.get_by_idx(mov.from);
        let target_piece = self.get_by_idx(mov.to);
        let move_is_capture = target_piece.0 != Piece::Empty;
        let (side_to_play, oponent_side) = (self.current_player(), !self.current_player());

        self.clear_bit(mov.from, p.0, side_to_play);
        if move_is_capture {
            if side_to_play == Color::White {
                if mov.to == Position(0+7*8) {
                    self.remove_castling_right(oponent_side, true);
                } else if mov.to == Position(7+7*8) {
                    self.remove_castling_right(oponent_side, false);
                }
            } else {
                if mov.to == Position(0+0*8) {
                    self.remove_castling_right(oponent_side, true);
                } else if mov.to == Position(7+0*8) {
                    self.remove_castling_right(oponent_side, false);
                }
            }

            self.clear_bit(mov.to, target_piece.0, target_piece.1);
        }

        self.set(mov.to, p.0, p.1);

        match mov.flag {
            3 => {
                if side_to_play == Color::White {
                    self.clear_bit(Position::new(0, 7), Piece::Rook, Color::White);
                    self.set(Position::new(0, 5), Piece::Rook, Color::White);
                    self.remove_castling_right(Color::White, false);
                    self.remove_castling_right(Color::White, true);
                } else {
                    self.clear_bit(Position::new(7, 7), Piece::Rook, Color::Black);
                    self.set(Position::new(7, 5), Piece::Rook, Color::Black);
                    self.remove_castling_right(Color::Black, false);
                    self.remove_castling_right(Color::Black, true);
                }
            }
            4 => {
                if side_to_play == Color::White {
                    self.clear_bit(Position::new(0, 0), Piece::Rook, Color::White);
                    self.set(Position::new(0, 3), Piece::Rook, Color::White);
                    self.remove_castling_right(Color::White, false);
                    self.remove_castling_right(Color::White, true);
                } else {
                    self.clear_bit(Position::new(7, 0), Piece::Rook, Color::Black);
                    self.set(Position::new(7, 3), Piece::Rook, Color::Black);
                    self.remove_castling_right(Color::Black, false);
                    self.remove_castling_right(Color::Black, true);
                }
            }
            5 => self.set(mov.to, Piece::Queen, p.1), // if promotion, set new piece on target instead of old one
            6 => self.set(mov.to, Piece::Rook, p.1),
            7 => self.set(mov.to, Piece::Bishop, p.1),
            8 => self.set(mov.to, Piece::Knight, p.1),
            _ => (),
        }

        if mov.flag == 1 {
            // move is en passant
            // TODO: better and more efficient board.clear_field to avoid use of board.get here
            self.clear_bit(
                Position::new(mov.from.rank(), mov.to.file()),
                Piece::Pawn,
                oponent_side,
            );
        } else if mov.flag == 2 {
            // move triggers en passant
            self.set_en_passant(mov.to.file() as u16);
        }

        if p.0 == Piece::King {
            if side_to_play == Color::White
                && (self.castle_white_short() || self.castle_white_long())
            {
                self.remove_castling_right(Color::White, false);
                self.remove_castling_right(Color::White, true);
            } else if side_to_play == Color::Black
                && (self.castle_black_short() || self.castle_black_long())
            {
                self.remove_castling_right(Color::Black, false);
                self.remove_castling_right(Color::Black, true);
            }
        } else if p.0 == Piece::Rook {
            if side_to_play == Color::White {
                if mov.from == Position::new(0, 0) {
                    self.remove_castling_right(Color::White, true);
                } else if mov.from == Position::new(0, 7) {
                    self.remove_castling_right(Color::White, false);
                }
            } else {
                if mov.from == Position::new(7, 0) {
                    self.remove_castling_right(Color::Black, true);
                } else if mov.from == Position::new(7, 7) {
                    self.remove_castling_right(Color::Black, false);
                }
            }
        }

        if (p.0 != Piece::Pawn) && !move_is_capture {
            self.half_moves += 1;

            if self.half_moves >= 100 {
                self.game_state = GameState::Draw;
            }

            let hash = self.compute_zobrist_hash();
            let mut repetition_count = 0;
            for i in 0..self.half_moves {
                if self.zobrist_history[i as usize] == hash {
                    repetition_count += 1;
                    if repetition_count == 2 {
                        self.game_state = GameState::Draw;
                        break;
                    }
                }
            }
            self.zobrist_history[self.half_moves as usize] = hash;
        } else {
            self.half_moves = 0;
            self.zobrist_history = [0; 101];
            if mov.flag == 2 {
                // if the new position includes a possible en passant, dont add it
                // to the zobrist history
                self.zobrist_history[0] = 0;
            } else {
                self.zobrist_history[0] = self.compute_zobrist_hash();
            }
        }

        let mut next_color_to_move = Color::Black;
        if self.current_player() == Color::Black {
            self.full_moves += 1;
            next_color_to_move = Color::White;
        }
        self.set_color_to_move(next_color_to_move);
        if mov.flag != 2 {
            // if en passant didnt just get triggered, reset it
            self.set_en_passant(15);
        }

        self.generate_total_bitboard(side_to_play);
        if move_is_capture || mov.flag == 1 {
            self.generate_total_bitboard(next_color_to_move);
        }
        self.generate_check_mask(side_to_play);

        self.generate_move_list();

        if self.move_list.len() == 0 {
            if self.king_attacker_count == 0 {
                self.game_state = GameState::Draw;
            } else {
                self.game_state = match next_color_to_move {
                    Color::White => GameState::BlackWins,
                    Color::Black => GameState::WhiteWins,
                }
            }
        }
    }
}
