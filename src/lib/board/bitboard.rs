use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Mul, Not};

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

impl BitAnd for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl Mul for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: Self) -> Self::Output {
        Self(self.0.wrapping_mul(other.0))
    }
}

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0;
    }
}

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> Self {
        BitBoard(!self.0)
    }
}

impl BitBoard {
    pub fn print(&self) {
        for row in (0..8).rev() {
            for col in 0..8 {
                print!(
                    "{} ",
                    (*self & BitBoard(1 << row * 8 + col) != BitBoard(0)) as u8
                );
            }
            println!();
        }
        println!();
    }

    pub fn count_set_bits(&self) -> u8 {
        let mut ret: u8 = 0;
        let mut n = self.0;
        while n != 0 {
            n &= n - 1;
            ret += 1;
        }
        ret
    }
}

pub struct BitBoardIter(pub u8, pub BitBoard);
impl IntoIterator for BitBoard {
    type Item = u8;
    type IntoIter = BitBoardIter;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        BitBoardIter(64, self)
    }
}
impl Iterator for BitBoardIter {
    type Item = u8;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.1 .0 == 0 {
            None
        } else {
            let sh = self.1 .0.leading_zeros() + 1;
            self.0 -= sh as u8;
            if sh >= 64 {
                self.1 .0 = 0;
            } else {
                self.1 .0 <<= sh;
            }
            Some(self.0)
        }
    }
}
