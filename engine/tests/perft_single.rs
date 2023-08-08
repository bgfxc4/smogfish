use smogfish::board::Board;

#[test]
fn perft_single() {
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(6, b, 7);
    assert_eq!(nodes, 119_060_324);
}

const NUMBER_TO_CHAR: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

fn perft(depth: u8, b: Board, start_depth: u8) -> u64 {
    let mut nodes: u64 = 0;

    if depth == 1 {
        return b.move_list.len() as u64;
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
