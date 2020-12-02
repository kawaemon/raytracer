use num_traits::{Float, NumOps};
use std::clone::Clone;
use std::marker::Copy;
use std::ops::{Add, Sub};

pub struct Vector3<T: NumOps> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: NumOps + Clone + Copy> Copy for Vector3<T> {}
impl<T: NumOps + Clone> Clone for Vector3<T> {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
        }
    }
}

impl<T: NumOps> Add for Vector3<T> {
    type Output = Vector3<T>;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T: NumOps> Sub for Vector3<T> {
    type Output = Vector3<T>;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: NumOps + Clone> Vector3<T> {
    pub fn scale(&self, n: T) -> Self {
        Self {
            x: self.x.clone() * n.clone(),
            y: self.y.clone() * n.clone(),
            z: self.z.clone() * n.clone(),
        }
    }

    pub fn dot(&self, other: &Self) -> T {
        self.x.clone() * other.x.clone()
            + self.y.clone() * other.y.clone()
            + self.z.clone() * other.z.clone()
    }
}

impl<T: Float> Vector3<T> {
    pub fn len(&self) -> T {
        T::sqrt(square(self.x) + square(self.y) + square(self.z))
    }

    pub fn normalize(&self) -> Self {
        self.scale(T::one() / self.len())
    }
}

fn square<T: NumOps + Clone>(n: T) -> T {
    n.clone() * n.clone()
}
