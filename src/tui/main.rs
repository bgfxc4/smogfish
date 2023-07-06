use colored::Colorize;
use std::io;

use smogfish::board::{Board, Position, Move};
use smogfish::board::helper::{Sides, Pieces};

pub fn main() {
    let mut board_buffer: [String; 8] = ["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()];
    let mut cursor_pos: Position = Position::new(3, 1);
     
    let mut b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    loop {
        fill_board_buffer(&b, &mut board_buffer);

        let mut possible_moves: Vec<Move> = vec![];
        if (b.get(&cursor_pos).1 == Sides::WHITE) == b.is_white_to_play() {
            get_possible_moves(&b, &cursor_pos, &mut possible_moves);
        }

        print_board_buffer(&board_buffer, &cursor_pos, &possible_moves);

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
        bb[row] =  "".to_string();
        for col in 0..8 {
            let p = b.get(&Position::new(col, row as u8));
            let c = match p.0 {
                Pieces::PAWN => "p",
                Pieces::KNIGHT => "n",
                Pieces::BISHOP => "b",
                Pieces::ROOK => "r",
                Pieces::QUEEN => "q",
                Pieces::KING => "k",
                _ => ".",
            };
            if p.1 == Sides::WHITE {
                bb[row].push_str(c.to_uppercase().as_str());
            } else {
                bb[row].push_str(c);
            }
        }
    }
}

fn print_board_buffer(bb: &[String; 8], cursor_pos: &Position, possible_moves: &Vec<Move>) {
    println!("   a b c d e f g h");
    println!("   ―――――――――――――――");
    for row in (0..8).rev() {
        print!("{}| ", row+1);
        let mut col = 0;
        for c in bb[row as usize].chars() {
            let mut to_print = c.to_string().normal();
            if col == cursor_pos.col && row == cursor_pos.row {
                to_print = to_print.bold().blue();
            }
            
            if possible_moves.iter().position(|m| m.to == Position::new(col, row)).is_some() {
                to_print = to_print.red();
            }

            print!("{} ", to_print);
            col += 1;
        }
        println!(" |{}", row+1);
    }
    println!("   ―――――――――――――――");
    println!("   a b c d e f g h");
}

fn handle_input(board: &mut Board, input: &String, cursor_pos: &mut Position, possible_moves: &Vec<Move>) {
    match input.trim() {
        "w" => cursor_pos.row += 1,
        "s" => cursor_pos.row -= 1,
        "d" => cursor_pos.col += 1,
        "a" => cursor_pos.col -= 1,
        cmd => {
            let mut chars = cmd.chars().into_iter();
            let col = chars.nth(0);
            let row = chars.nth(0);
            if (chars.count() == 0) && (col.unwrap().is_ascii()) && (row.unwrap().is_digit(10)) { // position got entered
                let pos = Position::new((col.unwrap() as u32 - 97) as u8, (row.unwrap().to_digit(10).unwrap() - 1) as u8);
                let mov = possible_moves.iter().find(|&m| &m.from == cursor_pos && m.to == pos);
                if mov.is_some() {
                    board.make_move(mov.unwrap());
                    return
                }
            }

            println!("Invalid command!");
        }
    }
}

fn get_possible_moves(b: &Board, cursor_pos: &Position, possible_moves: &mut Vec<Move>) {
    possible_moves.append(&mut b.get_all_possible_moves(cursor_pos));
}
