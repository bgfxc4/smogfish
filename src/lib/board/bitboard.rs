use std::ops::{BitAnd, BitOr, Mul, BitOrAssign, BitAndAssign};

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

