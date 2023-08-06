use colored::Colorize;
use std::io;

use smogfish::board::helper::{Color, GameState, Piece};
use smogfish::board::{Board, Move, Position};

pub fn main() {
    let mut board_buffer: [String; 8] = Default::default();
    let mut cursor_pos: Position = 3 + 1 * 8;

    let mut b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // let mut b = Board::new("R1K5/PP3q1r/2B5/8/8/5Q2/pp6/r5k1 b - - 0 1");
    // let mut b = Board::new("2k2Q2/8/8/8/1Q6/8/8/2K5 w - - 0 1");

    loop {
        if b.game_state != GameState::PLAYING {
            if b.game_state == GameState::DRAW {
                println!("The game ended by draw!");
                break;
            }
            if b.game_state == GameState::WHITE_WINS {
                println!("White wins!");
                break;
            }
            if b.game_state == GameState::BLACK_WINS {
                println!("Black wins!");
                break;
            }
        }

        fill_board_buffer(&b, &mut board_buffer);

        let mut possible_moves: Vec<Move> = vec![];
        if (b.get_by_idx(cursor_pos).1 == Color::White) == b.is_white_to_play() {
            get_possible_moves(&b, cursor_pos, &mut possible_moves);
        }

        print_board_buffer(&board_buffer, cursor_pos, &possible_moves);

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                handle_input(&mut b, &input, &mut cursor_pos, &possible_moves);
            }
            Err(error) => println!("error: {}", error),
        }
    }
}

fn fill_board_buffer(b: &Board, bb: &mut [String; 8]) {
    for row in (0..8).rev() {
        bb[row] = "".to_string();
        for col in 0..8 {
            let p = b.get_by_idx((row * 8 + col) as u8);
            let c = match p.0 {
                Piece::Pawn => "p",
                Piece::Knight => "n",
                Piece::Bishop => "b",
                Piece::Rook => "r",
                Piece::Queen => "q",
                Piece::King => "k",
                _ => ".",
            };
            if p.1 == Color::White {
                bb[row].push_str(c.to_uppercase().as_str());
            } else {
                bb[row].push_str(c);
            }
        }
    }
}

fn print_board_buffer(bb: &[String; 8], cursor_pos: Position, possible_moves: &Vec<Move>) {
    println!("   a b c d e f g h");
    println!("   ―――――――――――――――");
    for row in (0..8).rev() {
        print!("{}| ", row + 1);
        let mut col = 0;
        for c in bb[row as usize].chars() {
            let mut to_print = c.to_string().normal();
            if col + row * 8 == cursor_pos {
                to_print = to_print.bold().blue();
            }

            if possible_moves
                .iter()
                .position(|m| m.to == col + row * 8)
                .is_some()
            {
                to_print = to_print.red();
            }

            print!("{} ", to_print);
            col += 1;
        }
        println!(" |{}", row + 1);
    }
    println!("   ―――――――――――――――");
    println!("   a b c d e f g h");
}

fn handle_input(
    board: &mut Board,
    input: &String,
    cursor_pos: &mut Position,
    possible_moves: &Vec<Move>,
) {
    match input.trim() {
        "w" => *cursor_pos += 8,
        "s" => *cursor_pos -= 8,
        "d" => *cursor_pos += 1,
        "a" => *cursor_pos -= 1,
        cmd => {
            let mut chars = cmd.chars().into_iter();
            let col = chars.nth(0);
            let row = chars.nth(0);
            if row.is_some()
                && chars.count() == 0
                && (col.unwrap().is_ascii())
                && (row.unwrap().is_digit(10))
            {
                // position got entered
                let pos = (col.unwrap() as u32 - 97) as u8
                    + (row.unwrap().to_digit(10).unwrap() - 1) as u8 * 8;
                let mov = possible_moves
                    .iter()
                    .find(|&m| &m.from == cursor_pos && m.to == pos);
                match mov {
                    Some(m) => {
                        if m.flag >= 5 && m.flag <= 8 {
                            make_promotion_move(board, m);
                        } else {
                            board.make_move(m);
                        }

                        return;
                    }
                    None => {}
                }
            }

            println!("Invalid command!");
        }
    }
}

fn make_promotion_move(board: &mut Board, mov: &Move) {
    println!("To which piece do you want to promote? (Q/R/B/K)");
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => match input.chars().nth(0) {
            Some('Q') => board.make_move(&Move::new_with_flags(mov.from, mov.to, 5)),
            Some('R') => board.make_move(&Move::new_with_flags(mov.from, mov.to, 6)),
            Some('B') => board.make_move(&Move::new_with_flags(mov.from, mov.to, 7)),
            Some('K') => board.make_move(&Move::new_with_flags(mov.from, mov.to, 8)),
            _ => println!("Not a valid promotion"),
        },
        Err(error) => println!("error: {}", error),
    }
}

fn get_possible_moves(b: &Board, cursor_pos: Position, possible_moves: &mut Vec<Move>) {
    let mut a: Vec<Move> = b
        .move_list
        .iter()
        .filter(|m| m.from == cursor_pos)
        .map(|m| m.clone())
        .collect();
    possible_moves.append(&mut a);
}
