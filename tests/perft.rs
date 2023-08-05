use smogfish::board::{self, Board};

#[test]
fn run_perft() {
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // let b = Board::new("rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1");
    // let b = Board::new("rnbqkbnr/ppppppp1/7p/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
    let nodes = perft(4, b);
    println!("{} nodes", nodes);
}

fn perft(depth: u8, b: Board) -> u64 {
    let mut nodes: u64 = 0;
    
    if depth == 0 {
        return 1;
    }

    let moves = b.move_list.clone();
    for m in moves {
        let mut b_ = b.clone();
        b_.make_move(&m);
        let n = perft(depth - 1, b_);
        nodes += n;
        if depth == 4 {
            println!("move {} {} to {} {}, {} nodes", m.from.col, m.from.row, m.to.col, m.to.row, n);
        }
    }
    nodes
}
