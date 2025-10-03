use elliptic_curve::{ff::PrimeFieldBits, Group};

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{boxed::Box, string::String, vec::Vec};

// 娣诲姞 sum_of_products 妯″潡
pub mod sum_of_products;

// 娣诲姞瀵?serdes 妯″潡鐨勫鍑猴紙鏀惧湪鍚堥€備綅缃級
pub mod serdes;

/// 瀵逛换鎰忓疄鐜?Group 鐨勭被鍨嬶紝鎻愪緵鈥滄爣閲?鐐瑰鈥濈殑涔樺姞姹傚拰
pub trait SumOfProducts: Group {
    /// 璁＄畻 pairs 涓?(scalar_i * point_i) 鐨勫拰
    fn sum_of_products(pairs: &[(Self::Scalar, Self)]) -> Self;
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<G> SumOfProducts for G
where
    G: Group + zeroize::DefaultIsZeroes,
    G::Scalar: zeroize::DefaultIsZeroes + PrimeFieldBits,
{
    fn sum_of_products(pairs: &[(Self::Scalar, Self)]) -> Self {
        sum_of_products::sum_of_products_impl(pairs)
    }
}
