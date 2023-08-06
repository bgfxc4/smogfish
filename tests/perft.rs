use smogfish::board::Board;

#[test]
fn run_perft() {
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // let b = Board::new("rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1");
    // let b = Board::new("rnbqkbnr/ppppppp1/7p/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
    let nodes = perft(5, b);
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
        if depth == 5 {
            println!(
                "move {} {} to {} {}, {} nodes",
                m.from.file(),
                m.from.rank(),
                m.to.file(),
                m.to.rank(),
                n
            );
        }
    }
    nodes
}
