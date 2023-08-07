use super::{BitBoard, Board};
use std::ops::{Index, IndexMut, Not};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Color;
    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
    Empty = 6,
}

impl Piece {
    pub const ALL_NONEMPTY: [Piece; 6] = {
        use Piece::*;
        [Pawn, Knight, Bishop, Rook, Queen, King]
    };
}

#[derive(Debug, Clone)]
pub struct PieceBoards([[BitBoard; 6]; 2]);
impl Default for PieceBoards {
    #[inline(always)]
    fn default() -> Self {
        Self([[BitBoard(0); 6]; 2])
    }
}
impl Index<(Color, Piece)> for PieceBoards {
    type Output = BitBoard;
    #[inline(always)]
    fn index(&self, (color, piece): (Color, Piece)) -> &Self::Output {
        &self.0[color as usize][piece as usize]
    }
}
impl IndexMut<(Color, Piece)> for PieceBoards {
    #[inline(always)]
    fn index_mut(&mut self, (color, piece): (Color, Piece)) -> &mut Self::Output {
        &mut self.0[color as usize][piece as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position(pub u8);
impl Position {
    pub const fn new(rank: u8, file: u8) -> Self {
        Self(file + rank * 8)
    }
    pub fn file(self) -> u8 {
        self.0 % 8
    }
    pub fn rank(self) -> u8 {
        self.0 / 8
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    Playing,
    WhiteWins,
    BlackWins,
    Draw,
}

pub fn load_board_from_fen(board: &mut Board, fen: &str) -> Result<(), String> {
    board.flags = 0;
    board.pieces = PieceBoards::default();
    board.white_total = BitBoard(0);
    board.black_total = BitBoard(0);
    board.full_moves = 0;
    board.half_moves = 0;

    let groups: Vec<&str> = fen.split(" ").collect();
    if groups.len() != 6 {
        return Err("FEN parse error: The FEN string has to have 6 groups!".to_string());
    }

    //first group: piece positions
    let mut col: u8 = 0;
    let mut row: u8 = 7;
    for c in groups[0].chars() {
        let num = c.to_digit(10);
        if let Some(n) = num {
            col += n as u8;
            continue;
        }
        let pos = Position::new(row, col);
        #[rustfmt::skip] match c {
            '/' => {
                col = 0;
                row -= 1;
            },
            'P' => { board.set(pos, Piece::Pawn, Color::White); col += 1 },
            'N' => { board.set(pos, Piece::Knight, Color::White); col += 1 },
            'B' => { board.set(pos, Piece::Bishop, Color::White); col += 1 },
            'R' => { board.set(pos, Piece::Rook, Color::White); col += 1 },
            'Q' => { board.set(pos, Piece::Queen, Color::White); col += 1 },
            'K' => { board.set(pos, Piece::King, Color::White); col += 1 },
            'p' => { board.set(pos, Piece::Pawn, Color::Black); col += 1 },
            'n' => { board.set(pos, Piece::Knight, Color::Black); col += 1 },
            'b' => { board.set(pos, Piece::Bishop, Color::Black); col += 1 },
            'r' => { board.set(pos, Piece::Rook, Color::Black); col += 1 },
            'q' => { board.set(pos, Piece::Queen, Color::Black); col += 1 },
            'k' => { board.set(pos, Piece::King, Color::Black); col += 1 },
            _ => return Err("FEN parse error: illegal symbol in group 1".to_string())
        }
    }

    // second group: active color
    match groups[1] {
        "w" => board.set_color_to_move(Color::White),
        "b" => board.set_color_to_move(Color::Black),
        _ => return Err("FEN parse error: illegal symbol in group 2".to_string()),
    }

    // third group: castling rights
    if groups[2] != "-" {
        for c in groups[2].chars() {
            match c {
                'K' => board.set_castling_right(Color::White, false),
                'Q' => board.set_castling_right(Color::White, true),
                'k' => board.set_castling_right(Color::Black, false),
                'q' => board.set_castling_right(Color::Black, true),
                _ => return Err("FEN parse error: illegal symbol in group 3".to_string()),
            }
        }
    }

    // fourth group: en passant
    if groups[3] != "-" {
        let c = groups[3].chars().next().unwrap() as u32 - 97; // a -> 0; h -> 7
        if c > 7 {
            return Err("FEN parse error: illegal symbol in group 4".to_string());
        }
        board.set_en_passant(c as u16);
    } else {
        board.set_en_passant(15);
    }

    // fifth group: half moves
    let half_moves = groups[4].parse::<u8>();
    match half_moves {
        Ok(num) => board.half_moves = num,
        Err(_) => return Err("FEN parse error: illegal symbol in group 5".to_string()),
    }

    // sixth group: full moves
    let full_moves = groups[5].parse::<u16>();
    match full_moves {
        Ok(num) => board.full_moves = num,
        Err(_) => return Err("FEN parse error: illegal symbol in group 6".to_string()),
    }

    board.generate_total_bitboard(Color::White);
    board.generate_total_bitboard(Color::Black);
    board.generate_check_mask(!board.current_player());

    board.generate_move_list();

    return Ok(());
}
