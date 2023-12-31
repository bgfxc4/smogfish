use super::BitBoard;
use crate::board::helper::{Color, Position};

pub const DIRECTION_OFFSETS: [i8; 8] = [8, -8, -1, 1, 7, -7, 9, -9];
pub const NUM_SQUARES_TO_EDGE: [[i8; 8]; 64] = num_squares_to_edge();
pub const KNIGHT_ATTACKS: [BitBoard; 64] = knight_attacks();
pub const KING_ATTACKS: [BitBoard; 64] = king_attacks();
pub const KING_PAWN_ATTACKS: [[BitBoard; 64]; 2] = king_pawn_attacks();
/// 0: for short castle checks and pieces, 1: for long castle checks, 2: for long caslte_pieces
pub const KING_CASTLE_CHECKS: [[BitBoard; 3]; 2] = king_castle_checks();
pub const ZOBRIST_HASH_TABLE: [[u64; 12]; 64] = zobrist_hash_table();
pub const ZOBRIST_SPECIAL_KEYS: [u64; 5] = zobrist_special_keys();

/// stupid for-range implemention because const_trait_impl and iter are not usuable yet.
macro_rules! const_for {
    (for $i:ident in $from:literal..$to:literal { $($body:tt)* }) => {{
        let mut $i = $from;
        loop {
            if $i >= $to { break; }
            { $($body)* }
            $i += 1
        }
    }};
    (for $i:ident in [$($expr:expr),*] { $body:tt }) => {{
        { $({let $i = $expr; $body })* }
    }};

}
const fn min(a: i8, b: i8) -> i8 {
    if a < b {
        a
    } else {
        b
    }
}

const fn num_squares_to_edge() -> [[i8; 8]; 64] {
    let mut ret = [[0 as i8; 8]; 64];
    const_for!(for col in 0..8 {
        const_for!(for row in 0..8 {
            let up: i8 = 7 - row;
            let down: i8 = row;
            let left: i8 = col;
            let right: i8 = 7 - col;
            ret[(row * 8 + col) as usize] = [
                up,
                down,
                left,
                right,
                min(up, left),
                min(down, right),
                min(up, right),
                min(down, left),
            ]
        })
    });
    ret
}

const fn knight_attacks() -> [BitBoard; 64] {
    let mut ret = [BitBoard(0); 64];
    const_for!(for row in 0..8 {
        const_for!(for col in 0..8 {
            let square_idx = row * 8 + col;
            const_for!(for p in [
                (row + 2, col + 1),
                (row + 2, col - 1),
                (row - 2, col + 1),
                (row - 2, col - 1),
                (row + 1, col + 2),
                (row + 1, col - 2),
                (row - 1, col + 2),
                (row - 1, col - 2)
            ] {
                {
                    if p.0 <= 7 && p.0 >= 0 && p.1 <= 7 && p.1 >= 0 {
                        ret[square_idx as usize].0 |= 1 << (p.0 * 8 + p.1);
                    }
                }
            })
        })
    });
    ret
}

const fn king_attacks() -> [BitBoard; 64] {
    let mut ret = [BitBoard(0); 64];
    const_for!(for idx in 0..64 {
        const_for!(for dir_idx in 0..8 {
            if NUM_SQUARES_TO_EDGE[idx][dir_idx] != 0 {

                let dir = DIRECTION_OFFSETS[dir_idx as usize] as i8;
                let p = Position((idx as i8 + dir) as u8);

                ret[idx].0 |= 1 << (p.0);
            }
        })
    });
    ret
}

const fn king_pawn_attacks() -> [[BitBoard; 64]; 2] {
    let mut ret = [[BitBoard(0); 64]; 2];
    const_for!(for row in 0..8 {
        const_for!(for col in 0..8 {
            let square_idx = row * 8 + col;
            const_for!(for p in [
                (row + 1, col + 1, Color::White),
                (row + 1, col - 1, Color::White),
                (row - 1, col + 1, Color::Black),
                (row - 1, col - 1, Color::Black)
            ] {
                {
                    if p.0 <= 7 && p.0 >= 0 && p.1 <= 7 && p.1 >= 0 {
                        ret[p.2 as usize][square_idx as usize].0 |= 1 << (p.0 * 8 + p.1);
                    }
                }
            })
        })
    });
    ret
}

const fn king_castle_checks() -> [[BitBoard; 3]; 2] {
    [
        [BitBoard(96), BitBoard(12), BitBoard(14)],
        [BitBoard(6917529027641081856), BitBoard(864691128455135232), BitBoard(1008806316530991104)],
    ]
}

const fn xorshift(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    return x;
}

const fn zobrist_hash_table() -> [[u64; 12]; 64] {
    let mut ret = [[0; 12]; 64];
    let mut x = 31415;
    const_for!(for _x in 0..10 {
        x = xorshift(x)
    });
    const_for!(for i in 0..64 {
        const_for!(for p in 0..12 {
            x = xorshift(x);
            ret[i][p] = x;
        })
    });
    ret
}

const fn zobrist_special_keys() -> [u64; 5] {
    let mut ret = [0; 5];
    let mut x = 31415;
    const_for!(for _x in 0..80 {
        x = xorshift(x)
    });
    const_for!(for i in 0..5 {
        x = xorshift(x);
        ret[i] = x;
    });
    ret
}
