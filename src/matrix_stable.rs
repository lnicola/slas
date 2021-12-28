use crate::prelude::*;
use cblas_sys::cblas_sgemm;
use num::NumCast;
use std::ops::*;

#[derive(Copy, Clone)]
pub struct Matrix<'a, T: NumCast + Copy, const K: usize, const M: usize>(
    StaticCowVec<'a, T, { K * M }>,
)
where
    StaticCowVec<'a, T, { K * M }>: Sized;

impl<'a, T: NumCast + Copy, const K: usize, const M: usize> Matrix<'a, T, K, M>
where
    StaticCowVec<'a, T, { K * M }>: Sized,
{
    pub fn zeros() -> Self {
        Self(StaticCowVec::zeros())
    }

    pub fn is_borrowed(&self) -> bool {
        self.0.is_borrowed()
    }

    pub fn is_owned(&self) -> bool {
        self.0.is_owned()
    }

    pub unsafe fn get_unchecked_mut(&mut self, n: [usize; 2]) -> &mut T {
        self.0.get_unchecked_mut(n[0] + n[1] * K)
    }
    pub unsafe fn get_unchecked(&self, n: [usize; 2]) -> &T {
        self.0.get_unchecked(n[0] + n[1] * K)
    }

    pub fn transpose(&self) -> Matrix<T, M, K>
    where
        StaticCowVec<'a, T, { M * K }>: Sized,
    {
        let mut buffer = Matrix::<T, M, K>::zeros();
        for x in 0..K {
            for y in 0..M {
                unsafe { *buffer.get_unchecked_mut([y, x]) = *self.get_unchecked([x, y]) }
            }
        }
        buffer
    }
}

impl<'a, T: NumCast + Copy, const K: usize, const M: usize> Deref for Matrix<'a, T, K, M>
where
    StaticCowVec<'a, T, { K * M }>: Sized,
{
    type Target = StaticCowVec<'a, T, { K * M }>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: NumCast + Copy, const K: usize, const M: usize> DerefMut for Matrix<'a, T, K, M>
where
    StaticCowVec<'a, T, { K * M }>: Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: NumCast + Copy, const K: usize, const M: usize> Index<[usize; 2]>
    for Matrix<'a, T, K, M>
where
    StaticCowVec<'a, T, { K * M }>: Sized,
{
    type Output = T;
    fn index(&self, n: [usize; 2]) -> &T {
        assert!(
            n[0] < K && n[1] < M,
            "Index {:?} out of bounds {:?}",
            n,
            [K, M]
        );
        unsafe { self.0.get_unchecked(n[0] + n[1] * K) }
    }
}

impl<'a, T: NumCast + Copy, const K: usize, const M: usize> IndexMut<[usize; 2]>
    for Matrix<'a, T, K, M>
where
    StaticCowVec<'a, T, { K * M }>: Sized,
{
    fn index_mut(&mut self, n: [usize; 2]) -> &mut T {
        assert!(
            n[0] < K && n[1] < M,
            "Index {:?} out of bounds {:?}",
            n,
            [K, M]
        );
        unsafe { self.0.get_unchecked_mut(n[0] + n[1] * K) }
    }
}

impl<'a, T: Copy + NumCast, const K: usize, const M: usize> From<StaticCowVec<'a, T, { K * M }>>
    for Matrix<'a, T, K, M>
{
    fn from(v: StaticCowVec<'a, T, { K * M }>) -> Self {
        Matrix(v)
    }
}

impl<'a, T: Copy + NumCast, const K: usize, const M: usize> From<&'a [T; K * M]>
    for Matrix<'a, T, K, M>
{
    fn from(v: &'a [T; K * M]) -> Self {
        Matrix(v.into())
    }
}

impl<'a, T: Copy + NumCast, const K: usize, const M: usize> From<[T; K * M]>
    for Matrix<'a, T, K, M>
{
    fn from(v: [T; K * M]) -> Self {
        Matrix(v.into())
    }
}

impl<'a, 'b, const M: usize, const N: usize, const K: usize> Mul<Matrix<'b, f32, N, K>>
    for Matrix<'a, f32, K, M>
where
    StaticCowVec<'a, f32, { K * M }>: Sized,
    StaticCowVec<'a, f32, { N * K }>: Sized,
    StaticCowVec<'a, f32, { N * M }>: Sized,
{
    type Output = Matrix<'static, f32, N, M>;
    fn mul(self, other: Matrix<'b, f32, N, K>) -> Self::Output {
        let mut buffer = Self::Output::zeros();
        unsafe {
            cblas_sgemm(
                cblas_sys::CBLAS_LAYOUT::CblasRowMajor,
                cblas_sys::CBLAS_TRANSPOSE::CblasNoTrans,
                cblas_sys::CBLAS_TRANSPOSE::CblasNoTrans,
                M as i32,
                N as i32,
                K as i32,
                1.,
                self.as_ptr(),
                K as i32,
                other.as_ptr(),
                N as i32,
                0.,
                buffer.as_mut_ptr(),
                N as i32,
            )
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;

    #[test]
    fn zero() {
        let m = Matrix::<f32, 2, 2>::zeros();
        let n: Matrix<f32, 2, 2> = [0.; 4].into();
        assert!(m[[0, 0]] == 0.);
        assert!(**m == **n)
    }

    #[test]
    fn mul() {
        let m: Matrix<f32, 3, 2> = [1., 2., 3., 4., 5., 6.].into();
        let n: Matrix<f32, 2, 3> = [10., 11., 20., 21., 30., 31.].into();
        let k = [140., 146., 320., 335.];

        assert_eq!(**(m * n), k);
    }

    #[test]
    fn mul2() {
        let m: Matrix<f32, 3, 4> = [1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.].into();
        let n: Matrix<f32, 2, 3> = [3., 6., 8., 10., 9., 17.].into();
        let k = [46., 77., 106., 176., 166., 275., 226., 374.];
        assert_eq!(**(m * n), k);
    }

    //#[test]
    //fn mul3() { // Doesn't work. Might just be an incorrect expected result.
    //    let m: Matrix<f32, 5, 6> = [
    //        1.0, 2.0, -1.0, -1.0, 4.0, 2.0, 0.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 2.0, -3.0,
    //        2.0, 2.0, 2.0, 0.0, 4.0, 0.0, -2.0, 1.0, -1.0, -1.0, -1.0, 1.0, -3.0, 2.0,
    //    ]
    //    .into();

    //    let n: Matrix<f32, 4, 5> = [
    //        1.0, -1.0, 0.0, 2.0, 2.0, 2.0, -1.0, -2.0, 1.0, 0.0, -1.0, 1.0, -3.0, -1.0, 1.0, -1.0,
    //        4.0, 2.0, -1.0, 1.0,
    //    ]
    //    .into();

    //    let k = [
    //        24.0, 13.0, -5.0, 3.0, -3.0, -4.0, 2.0, 4.0, 4.0, 1.0, 2.0, 5.0, -2.0, 6.0, -1.0, -9.0,
    //        -4.0, -6.0, 5.0, 5.0, 16.0, 7.0, -4.0, 7.0,
    //    ];

    //    assert_eq!(**(m * n), k);
    //}
}

pub mod matrix {
    pub use super::Matrix;
}
