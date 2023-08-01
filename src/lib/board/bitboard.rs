use std::ops::{BitAnd, BitOr, Mul, BitOrAssign, BitAndAssign, Not};

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
                print!("{} ", if *self & BitBoard(1 << row*8+col) == BitBoard(0) { 0 } else { 1 });
            }
            println!();
        }
        println!();
    }
}
