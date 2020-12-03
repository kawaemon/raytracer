use crate::intersect::{Intersectable, Intersection};
use crate::material::Material;
use crate::ray::Ray;
use crate::vector::Vector3;

pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub material: Material,
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let v = ray.origin - self.center;
        let b = ray.dir.dot(&v);
        let c = v.dot(&v) - (self.radius * self.radius);
        let d = (b * b) - c;

        if d >= 0.0 {
            let s = d.sqrt();
            let mut t = -b - s;

            if t <= 0.0 {
                t = -b + s
            }

            if 0.0 < t {
                let point = ray.origin + ray.dir.scale(t);
                let normal = (point - self.center).normalize();
                return Some(Intersection {
                    distance: t,
                    point,
                    normal,
                    material: self.material.clone(),
                });
            }
        }

        None
    }
}
