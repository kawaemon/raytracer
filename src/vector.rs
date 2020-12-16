use std::clone::Clone;
use std::marker::Copy;
use std::ops::{Add, AddAssign, Neg, Sub};

#[derive(Clone, Copy, Default)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Vector3 {
    pub fn scale(&self, n: f64) -> Self {
        Self {
            x: self.x * n,
            y: self.y * n,
            z: self.z * n,
        }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        *self - normal.scale(2.0 * self.dot(&normal))
    }

    pub fn len(&self) -> f64 {
        f64::sqrt(square(self.x) + square(self.y) + square(self.z))
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
        #[cfg(debug_assertions)]
        let mut rng = || {
            let value = rng();
            debug_assert!((-1.0..=1.0).contains(&value));
            value
        };

        loop {
            let mut dir = Vector3 {
                x: rng(),
                y: rng(),
                z: rng(),
            };

            if dir.len() < 1.0 {
                dir = dir.normalize();
                if dir.dot(self) < 0.0 {
                    dir = -dir;
                }
                break dir;
            }
        }
    }
}

fn square(n: f64) -> f64 {
    n * n
}
