use smogfish::board::Board;
use smogfish::board::helper::{Sides, Pieces};

pub fn main() {
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    print_board(&b);
}

fn print_board(b: &Board) {
    for row in 0..8 {
        for col in 0..8 {
            let p = b.get(col, 7 - row);
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
                print!("{0} ", c.to_uppercase());
            } else {
                print!("{0} ", c);
            }
        }
        println!();
    }
}
