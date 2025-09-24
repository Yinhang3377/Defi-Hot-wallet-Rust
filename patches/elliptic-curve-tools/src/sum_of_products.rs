use elliptic_curve::{ff::PrimeFieldBits, Group};

/// 对任意实现 Group 的类型，提供“标量-点对”的乘加求和
pub trait SumOfProducts: Group {
    /// 计算 pairs 中 (scalar_i * point_i) 的和
    fn sum_of_products(pairs: &[(Self::Scalar, Self)]) -> Self
    where
        // 为 multiexp 的约束补齐：点与标量都需要 Zeroize(DefaultIsZeroes)，标量还需 PrimeFieldBits
        Self: zeroize::DefaultIsZeroes,
        Self::Scalar: zeroize::DefaultIsZeroes + PrimeFieldBits,
    {
        multiexp::multiexp::<Self>(pairs)
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<G> SumOfProducts for G
where
    // 这里继续要求 G: DefaultIsZeroes（与上面方法约束一致）
    G: Group + zeroize::DefaultIsZeroes,
{}
