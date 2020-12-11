use crate::intersect::{Intersectable, Intersection};
use crate::material::Material;
use crate::ray::Ray;
use crate::vector::Vector3;

pub struct Plane {
    pub normal: Vector3,
    pub distance: f64,
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vector3, normal: Vector3, material: Material) -> Self {
        let normal = normal.normalize();

        Self {
            normal,
            distance: -point.dot(&normal),
            material,
        }
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let v = self.normal.dot(&ray.dir);
        let t = -(self.normal.dot(&ray.origin) + self.distance) / v;

        if 0.0 < t {
            return Some(Intersection {
                distance: t,
                point: ray.origin + ray.dir.scale(t),
                normal: self.normal,
                material: self.material.clone(),
            });
        }

        None
    }
}
