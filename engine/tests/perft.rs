use smogfish::board::Board;

#[test]
#[ignore]
fn perft_manually() {
    let args: Vec<String> = std::env::args().collect();
    let b = match args.len() > 5 {
        true => Board::new(args[5].as_str()),
        false => Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    };
    let nodes = match args.len() > 6 {
        true => perft(args[6].parse().unwrap(), b, args[6].parse().unwrap()),
        false => perft(5, b, 5)
    };
    println!("{} nodes", nodes);
}

#[test]
fn perft_startpos_5() {
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(5, b, 7);
    assert_eq!(nodes, 4_865_609);
}

#[test]
fn perft_startpos_6() {
    let b = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let nodes = perft(6, b, 7);
    assert_eq!(nodes, 119_060_324);
}

#[test]
fn perft_kiwipete_4() {
    let b = Board::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let nodes = perft(4, b, 6);
    assert_eq!(nodes, 4_085_603);
}

#[test]
fn perft_kiwipete_5() {
    let b = Board::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let nodes = perft(5, b, 6);
    assert_eq!(nodes, 193_690_690);
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
