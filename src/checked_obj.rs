use crate::{
    intersect::{Intersectable, Intersection},
    material::Material,
    ray::Ray,
};

// "柄が"チェック柄
pub struct CheckedObject<T: Intersectable> {
    pub object: T,
    pub grid_width: f64,
    pub alt_material: Material,
}

impl<T: Intersectable> Intersectable for CheckedObject<T> {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        self.object.intersect(ray).map(|mut intersection| {
            let i = (intersection.point.x / self.grid_width).round()
                + (intersection.point.y / self.grid_width).round()
                + (intersection.point.z / self.grid_width).round();

            if i % 2.0 == 0.0 {
               intersection.material = self.alt_material.clone()
            }

            intersection
        })
    }
}
