use rand::Rng;
use std::cmp;

use crate::board::helper::Color;

use super::BitBoard;

lazy_static! {
    pub static ref PRECOMPUTED_LOOKUPS: PrecomputedLookups = PrecomputedLookups {
        NUM_SQUARES_TO_EDGE: precompute_num_squares_to_edge(),
        DIRECTION_OFFSETS: [8, -8, -1, 1, 7, -7, 9, -9],
        KNIGHT_ATTACKS: precompute_knight_attacks(),
        KING_PAWN_ATTACKS: precompute_king_pawn_attacks(),
        KING_CASTLE_CHECKS: precompute_king_castle_checks(),
        ZOBRIST_HASH_TABLE: init_zobrist_hash_table(),
        // black to move, white short castle, white long castle, black short castle, black long castle
        ZOBRIST_SPECIAL_KEYS: init_zobrist_special_keys(),
    };
}
#[allow(non_snake_case)]
pub struct PrecomputedLookups {
    pub NUM_SQUARES_TO_EDGE: [[i8; 8]; 64],
    pub DIRECTION_OFFSETS: [i8; 8],
    pub KNIGHT_ATTACKS: [BitBoard; 64],
    pub KING_PAWN_ATTACKS: [[BitBoard; 64]; 2],
    pub KING_CASTLE_CHECKS: [[BitBoard; 2]; 2],
    pub ZOBRIST_HASH_TABLE: [[u64; 12]; 64],
    pub ZOBRIST_SPECIAL_KEYS: [u64; 5],
}

fn precompute_num_squares_to_edge() -> [[i8; 8]; 64] {
    println!("Precomputing num squares to edge...");
    let mut ret = [[0 as i8; 8]; 64];
    for col in 0..8 {
        for row in 0..8 {
            let up: i8 = 7 - row;
            let down: i8 = row;
            let left: i8 = col;
            let right: i8 = 7 - col;
            ret[(row * 8 + col) as usize] = [
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
            let square_idx = row * 8 + col;
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
                ret[square_idx as usize] |= BitBoard(1 << (p.0 * 8 + p.1));
            }
        }
    }
    println!("Done!");
    ret
}

fn precompute_king_pawn_attacks() -> [[BitBoard; 64]; 2] {
    println!("Precomputing king pawn attacks...");
    let mut ret = [[BitBoard(0); 64]; 2];

    for row in 0..8 as i8 {
        for col in 0..8 as i8 {
            let square_idx = row * 8 + col;
            let possible_attacks = [
                (row + 1, col + 1, Color::White),
                (row + 1, col - 1, Color::White),
                (row - 1, col + 1, Color::Black),
                (row - 1, col - 1, Color::Black),
            ];
            for p in possible_attacks {
                if p.0 > 7 || p.0 < 0 || p.1 > 7 || p.1 < 0 {
                    continue;
                }
                ret[p.2 as usize][square_idx as usize] |= BitBoard(1 << (p.0 * 8 + p.1));
            }
        }
    }
    println!("Done!");
    ret
}

fn precompute_king_castle_checks() -> [[BitBoard; 2]; 2] {
    [
        [BitBoard(96), BitBoard(12)],
        [BitBoard(6917529027641081856), BitBoard(864691128455135232)],
    ]
}

fn init_zobrist_hash_table() -> [[u64; 12]; 64] {
    println!("Precomputing zobrist table...");
    let mut rng = rand::thread_rng();
    let mut ret = [[0; 12]; 64];
    for i in 0..64 {
        for p in 0..12 {
            ret[i][p] = rng.gen();
        }
    }
    println!("Done!");
    ret
}

fn init_zobrist_special_keys() -> [u64; 5] {
    println!("Precomputing special keys...");
    let mut rng = rand::thread_rng();
    println!("Done!");
    [rng.gen(), rng.gen(), rng.gen(), rng.gen(), rng.gen()]
}
