use crate::intersect::{Intersectable, Intersection};
use crate::material::Material;
use crate::ray::Ray;
use crate::spectrum::Spectrum;
use crate::vector::Vector3;

pub struct TexturedObj<T, I>
where
    T: Intersectable,
    I: Fn(u32, u32) -> Spectrum,
{
    pub object: T,
    pub image: I,
    pub texture_size: f64,
    pub image_width: u32,
    pub image_height: u32,
    pub origin: Vector3,
    pub u_direction: Vector3,
    pub v_direction: Vector3,
}

impl<T, I> Intersectable for TexturedObj<T, I>
where
    T: Intersectable,
    I: Fn(u32, u32) -> Spectrum,
{
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let intersection = self.object.intersect(ray);

        if let Some(intersection) = intersection {
            let u = {
                let u =
                    (intersection.point - self.origin).dot(&self.u_direction) / self.texture_size;
                ((u - u.floor()) * self.image_width as f64).floor()
            };

            let v = {
                let v =
                    -(intersection.point - self.origin).dot(&self.v_direction) / self.texture_size;
                ((v - v.floor()) * self.image_height as f64).floor()
            };

            let color = (self.image)(u as _, v as _);

            return Some(Intersection {
                material: Material {
                    diffuse: color * intersection.material.diffuse,
                    ..intersection.material
                },
                ..intersection
            });
        }

        None
    }
}
