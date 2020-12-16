use std::{
    arch::x86_64::{
        __m256d, _mm256_add_pd, _mm256_div_pd, _mm256_mul_pd, _mm256_set1_pd, _mm256_set_pd,
        _mm256_sub_pd,
    },
    convert::AsRef,
    ops::{Add, AddAssign, Mul, Sub},
};

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[repr(C)]
pub struct f64x3 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub _padding: f64,
}

impl From<__m256d> for f64x3 {
    fn from(t: __m256d) -> Self {
        unsafe { std::mem::transmute(t) }
    }
}

impl Into<__m256d> for f64x3 {
    fn into(self) -> __m256d {
        unsafe { std::mem::transmute(self) }
    }
}

impl f64x3 {
    pub fn sum(&self) -> f64 {
        unsafe { (&*(&self as *const _ as *const [f64; 3])).into_iter().sum() }
    }
}

impl Add for f64x3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_add_pd(self.into(), rhs.into()).into() }
    }
}

impl AddAssign for f64x3 {
    fn add_assign(&mut self, rhs: Self) {
        unsafe { *self = _mm256_add_pd((*self).into(), rhs.into()).into() }
    }
}

impl Sub for f64x3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_sub_pd(self.into(), rhs.into()).into() }
    }
}

impl Mul for f64x3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_mul_pd(self.into(), rhs.into()).into() }
    }
}

impl Mul<f64> for f64x3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        unsafe {
            let rhs = _mm256_set1_pd(rhs);
            _mm256_mul_pd(self.into(), rhs).into()
        }
    }
}

#[test]
fn test() {
    let f1 = f64x3 {
        a: 1.0,
        b: 2.0,
        c: 3.0,
        _padding: 0.0,
    };

    let f2 = f64x3 {
        a: 1.0,
        b: 2.0,
        c: 3.0,
        _padding: 0.0,
    };

    let result = f1 + f2;

    assert_eq!(
        result,
        f64x3 {
            a: 2.0,
            b: 4.0,
            c: 6.0,
            _padding: 0.0
        }
    )
}
