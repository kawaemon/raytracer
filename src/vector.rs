use std::clone::Clone;
use std::marker::Copy;
use std::ops::{Add, AddAssign, Neg, Sub, Deref};
use crate::simd::f64x3;

#[derive(Clone, Copy, Default)]
pub struct Vector3 {
    inner: f64x3
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            inner: self.inner + other.inner
        }
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, other: Self) {
        self.inner += other.inner;
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            inner: self.inner - other.inner
        }
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            inner: f64x3 {
                a: -self.inner.a,
                b: -self.inner.b,
                c: -self.inner.c,
                ..f64x3::default()
            }
        }
    }
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: f64x3 {
                a: x,
                b: y,
                c: z,
                ..f64x3::default()
            }
        }
    }

    pub fn x(&self) -> f64 {
        self.inner.a
    }

    pub fn y(&self) -> f64 {
        self.inner.b
    }

    pub fn z(&self) -> f64 {
        self.inner.c
    }

    pub fn scale(&self, n: f64) -> Self {
        Self {
            inner: self.inner * n
        }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        (self.inner * other.inner).sum()
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        *self - normal.scale(2.0 * self.dot(&normal))
    }

    pub fn len(&self) -> f64 {
        (self.inner * self.inner).sum().sqrt()
    }

    pub fn normalize(&self) -> Self {
        self.scale(1.0 / self.len())
    }

    pub fn refract(&self, normal: Self, eta: f64) -> Self {
        let dot = self.dot(&normal);
        let d = 1.0 - square(eta) * (1.0 - square(dot));

        if 0.0 < d {
            let a = (*self - normal.scale(dot)).scale(eta);
            let b = normal.scale(d.sqrt());
            return a - b;
        }

        self.reflect(&normal)
    }

    /// rng must produce numbers which is in -1.0..1.0
    pub fn random_hemisphere(&self, mut rng: impl FnMut() -> f64) -> Self {
        let mut safer_rng = || {
            let value = rng();
            debug_assert!((-1.0..1.0).contains(&value));
            value
        };

        loop {
            let mut dir = Vector3::new(
                 safer_rng(),
                 safer_rng(),
                 safer_rng(),
            );

            if dir.len() < 1.0 {
                dir = dir.normalize();
                if dir.dot(self) < 0.0 {
                    dir = -dir;
                }
            }

            break dir;
        }
    }
}

fn square(n: f64) -> f64 {
    n * n
}
