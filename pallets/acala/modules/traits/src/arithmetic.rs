pub use num_traits::{
    Bounded,
    CheckedAdd,
    CheckedDiv,
    CheckedMul,
    CheckedShl,
    CheckedShr,
    CheckedSub,
    One,
    Signed,
    Zero
};

use sp_std::{
    self,
    ops::{
        Add,
        AddAssign,
        Div,
        DivAssign,
        Mul,
        MulAssign,
        Rem,
        RemAssign,
        Shl,
        Shr,
        Sub,
        SubAssign
    }
};

pub trait SimpleArithmetic:
	Zero
	+ One
	+ From<u8>
	+ From<u16>
	+ From<u32>
	+ TryInto<u8>
	+ TryInto<u16>
	+ TryInto<u32>
	+ TryFrom<u64>
	+ TryInto<u64>
	+ TryFrom<u128>
	+ TryInto<u128>
	+ Add<Self, Output = Self>
	+ AddAssign<Self>
	+ Sub<Self, Output = Self>
	+ SubAssign<Self>
	+ Mul<Self, Output = Self>
	+ MulAssign<Self>
	+ Div<Self, Output = Self>
	+ DivAssign<Self>
	+ Rem<Self, Output = Self>
	+ RemAssign<Self>
	+ Shl<u32, Output = Self>
	+ Shr<u32, Output = Self>
	+ CheckedShl
	+ CheckedShr
	+ CheckedAdd
	+ CheckedSub
	+ CheckedMul
	+ CheckedDiv
	+ PartialOrd<Self>
	+ Ord
	+ Bounded
	+ Sized
{
}

impl<
		T: Zero
			+ One
			+ From<u8>
			+ From<u16>
			+ From<u32>
			+ TryInto<u8>
			+ TryInto<u16>
			+ TryInto<u32>
			+ TryFrom<u64>
			+ TryInto<u64>
			+ TryFrom<u128>
			+ TryInto<u128>
			+ Add<Self, Output = Self>
			+ AddAssign<Self>
			+ Sub<Self, Output = Self>
			+ SubAssign<Self>
			+ Mul<Self, Output = Self>
			+ MulAssign<Self>
			+ Div<Self, Output = Self>
			+ DivAssign<Self>
			+ Rem<Self, Output = Self>
			+ RemAssign<Self>
			+ Shl<u32, Output = Self>
			+ Shr<u32, Output = Self>
			+ CheckedShl
			+ CheckedShr
			+ CheckedAdd
			+ CheckedSub
			+ CheckedMul
			+ CheckedDiv
			+ PartialOrd<Self>
			+ Ord
			+ Bounded
			+ Sized,
	> SimpleArithmetic for T
{
}