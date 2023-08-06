use super::{Board, BitBoard, Position};

pub struct Sides;
impl Sides {
    pub const WHITE: u8 = 0;
    pub const BLACK: u8 = 1;
}

pub struct Pieces;
impl Pieces {
    pub const PAWN: u8 = 0;
    pub const KNIGHT: u8 = 1;
    pub const BISHOP: u8 = 2;
    pub const ROOK: u8 = 3;
    pub const QUEEN: u8 = 4;
    pub const KING: u8 = 5;
    pub const EMPTY: u8 = 6;
}

pub struct GameState;
impl GameState {
    pub const PLAYING: u8 = 0;
    pub const WHITE_WINS: u8 = 1;
    pub const BLACK_WINS: u8 = 2;
    pub const DRAW: u8 = 3;
}

pub fn load_board_from_fen(board: &mut Board, fen: &str) -> Result<(), String> {
    board.flags = 0;
    board.pieces = [[BitBoard(0); 6]; 2];
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
        let pos = col+8*row;
        match c {
            '/' => {
                col = 0;
                row -= 1;
            },
            'P' => { board.set(pos, Pieces::PAWN, Sides::WHITE); col += 1 },
            'N' => { board.set(pos, Pieces::KNIGHT, Sides::WHITE); col += 1 },
            'B' => { board.set(pos, Pieces::BISHOP, Sides::WHITE); col += 1 },
            'R' => { board.set(pos, Pieces::ROOK, Sides::WHITE); col += 1 },
            'Q' => { board.set(pos, Pieces::QUEEN, Sides::WHITE); col += 1 },
            'K' => { board.set(pos, Pieces::KING, Sides::WHITE); col += 1 },
            'p' => { board.set(pos, Pieces::PAWN, Sides::BLACK); col += 1 },
            'n' => { board.set(pos, Pieces::KNIGHT, Sides::BLACK); col += 1 },
            'b' => { board.set(pos, Pieces::BISHOP, Sides::BLACK); col += 1 },
            'r' => { board.set(pos, Pieces::ROOK, Sides::BLACK); col += 1 },
            'q' => { board.set(pos, Pieces::QUEEN, Sides::BLACK); col += 1 },
            'k' => { board.set(pos, Pieces::KING, Sides::BLACK); col += 1 },
            _ => return Err("FEN parse error: illegal symbol in group 1".to_string())
        } 
    }

    // second group: active color
    match groups[1] {
        "w" => board.set_color_to_move(Sides::WHITE),
        "b" => board.set_color_to_move(Sides::BLACK),
        _ => return Err("FEN parse error: illegal symbol in group 2".to_string())
    }

    // third group: castling rights
    if groups[2] != "-" {
        for c in groups[2].chars() {
            match c {
                'K' => board.set_castling_right(Sides::WHITE, false),
                'Q' => board.set_castling_right(Sides::WHITE, true),
                'k' => board.set_castling_right(Sides::BLACK, false),
                'q' => board.set_castling_right(Sides::BLACK, true),
                _ => return Err("FEN parse error: illegal symbol in group 3".to_string())
            }
        }
    }

    // fourth group: en passant
    if groups[3] != "-" {
        let c = groups[3].chars().next().unwrap() as u32 - 97; // a -> 0; h -> 7
        if c > 7 {
            return Err("FEN parse error: illegal symbol in group 4".to_string())
        }
        board.set_en_passant(c as u16);
    }

    // fifth group: half moves
    let half_moves = groups[4].parse::<u8>();
    match half_moves {
        Ok(num) => board.half_moves = num,
        Err(_) => return Err("FEN parse error: illegal symbol in group 5".to_string())
    }


    // sixth group: full moves
    let full_moves = groups[5].parse::<u16>();
    match full_moves {
        Ok(num) => board.full_moves = num,
        Err(_) => return Err("FEN parse error: illegal symbol in group 6".to_string())
    }

    board.generate_total_bitboard(Sides::WHITE);
    board.generate_total_bitboard(Sides::BLACK);
    board.generate_check_mask(if board.is_white_to_play() { Sides::BLACK } else { Sides::WHITE });

    board.generate_move_list();

    return Ok(());
}
