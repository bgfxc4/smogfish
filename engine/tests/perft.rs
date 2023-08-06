use smogfish::board::Board;

#[test]
fn run_perft() {
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(5, b, 5);
    println!("{} nodes", nodes);
}

const NUMBER_TO_CHAR: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

fn perft(depth: u8, b: Board, start_depth: u8) -> u64 {
    let mut nodes: u64 = 0;

    if depth == 0 {
        return 1;
    }

    let moves = b.move_list.clone();
    for m in moves {
        let mut b_ = b.clone();
        b_.make_move(&m);
        let n = perft(depth - 1, b_, start_depth);
        nodes += n;
        if depth == start_depth {
            println!(
                "{}{}{}{}: {}",
                NUMBER_TO_CHAR[m.from.file() as usize],
                m.from.rank() + 1,
                NUMBER_TO_CHAR[m.to.file() as usize],
                m.to.rank() + 1,
                n
            );
        }
    }
    nodes
}
