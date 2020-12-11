use crate::vector::Vector3;

const EPSILON: f64 = 0.001;

pub struct Ray {
    pub origin: Vector3,
    pub dir: Vector3,
}

impl Ray {
    pub fn new(origin: Vector3, dir: Vector3) -> Self {
        let dir = dir.normalize();
        let origin = origin + dir.scale(EPSILON);
        Self {
            origin,
            dir: dir.normalize(),
        }
    }
}
