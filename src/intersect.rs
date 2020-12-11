use crate::material::Material;
use crate::ray::Ray;
use crate::vector::Vector3;

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

pub struct Intersection {
    pub distance: f64,
    pub point: Vector3,
    pub normal: Vector3, // 法線
    pub material: Material,
}
