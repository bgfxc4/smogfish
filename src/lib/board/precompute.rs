use std::cmp;
use super::BitBoard;

lazy_static! {
    pub static ref PRECOMPUTED_LOOKUPS: PrecomputedLookups = PrecomputedLookups {
        NUM_SQUARES_TO_EDGE: precompute_num_squares_to_edge(),
        DIRECTION_OFFSETS: [8, -8, -1, 1, 7, -7, 9, -9],
        KNIGHT_ATTACKS: precompute_knight_attacks(),
    };
}
#[allow(non_snake_case)]
pub struct PrecomputedLookups {
    pub NUM_SQUARES_TO_EDGE: [[i8; 8]; 64],
    pub DIRECTION_OFFSETS: [i8; 8],
    pub KNIGHT_ATTACKS: [BitBoard; 64],
}

fn precompute_num_squares_to_edge() -> [[i8; 8]; 64] {
    println!("Precomputing num squares to edge...");
    let mut ret = [[0 as i8; 8]; 64];
    for col in 0 .. 8 {
        for row in 0 .. 8 {
            let up: i8 = 7 - row;
            let down: i8 = row;
            let left: i8 = col;
            let right: i8 = 7 - col;
            ret[(row*8+col) as usize] = [
                up,
                down,
                left,
                right,
                cmp::min(up, left),
                cmp::min(down, right),
                cmp::min(up, right),
                cmp::min(down, left),
            ]
        }
    }
    println!("Done!");
    ret
}

fn precompute_knight_attacks() -> [BitBoard; 64] {
    println!("Precomputing knight attacks...");
    let mut ret = [BitBoard(0); 64];

    for row in 0..8 as i8 {
        for col in 0..8 as i8 {
            let square_idx = row*8 + col;
            let possible_attacks = [
                (row + 2, col + 1),
                (row + 2, col - 1),
                (row - 2, col + 1),
                (row - 2, col - 1),
                (row + 1, col + 2),
                (row + 1, col - 2),
                (row - 1, col + 2),
                (row - 1, col - 2),
            ];
            for p in possible_attacks {
                if p.0 > 7 || p.0 < 0 || p.1 > 7 || p.1 < 0 {
                    continue;
                }
                ret[square_idx as usize] |= BitBoard(1 << (p.0*8 + p.1));
            }
        } 
    } 
    println!("Done!");
    ret
}
