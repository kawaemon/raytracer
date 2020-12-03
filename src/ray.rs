use crate::vector::Vector3;

pub struct Ray {
    pub origin: Vector3<f64>,
    pub dir: Vector3<f64>,
}

impl Ray {
    pub fn new(origin: Vector3<f64>, dir: Vector3<f64>) -> Self {
        Self {
            origin,
            dir: dir.normalize(),
        }
    }
}
