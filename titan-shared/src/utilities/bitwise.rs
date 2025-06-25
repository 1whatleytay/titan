use num_traits::{Unsigned, WrappingShl, WrappingShr};
use std::ops::{BitAnd, BitOr, Not};

// You must cast to signed type.
#[inline]
pub fn sign_extend<
    T: Sized
        + Unsigned
        + Copy
        + WrappingShl<Output = T>
        + WrappingShr<Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + Not<Output = T>
>(
    value: T,
    bit_size: u32,
) -> T {
    let bits = size_of::<T>() * 8;

    let cut_off = bits as u32 - bit_size;

    let mask = T::zero().not().wrapping_shl(cut_off).wrapping_shr(cut_off);

    if value.bitand(T::one().wrapping_shl(bit_size - 1)).is_zero() {
        // Sign bit is not set!

        value.bitand(mask) // zero out post-sign bits
    } else {
        // Sign bit is set!

        value.bitor(mask.not()) // set all post-sign bits
    }
}

#[inline]
pub fn pick_bits<
    T: Sized
        + Unsigned
        + WrappingShl<Output = T>
        + WrappingShr<Output = T>
        + BitAnd<Output = T>
        + Not<Output = T>,
>(
    value: T,
    start: u32,
    count: u32,
) -> T {
    let bits = size_of::<T>() * 8;

    let cut_off = bits as u32 - count;

    // Hopefully the type is unsigned!
    let mask = T::zero().not().wrapping_shl(cut_off).wrapping_shr(cut_off);

    value.bitand(mask.wrapping_shl(start)).wrapping_shr(start)
}
