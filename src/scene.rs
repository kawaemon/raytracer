use crate::intersect::{Intersectable, Intersection};
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::spectrum::{self, Spectrum};
use crate::vector::Vector3;

pub struct Scene {
    objects: Vec<Box<dyn Intersectable>>,
    lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            lights: vec![],
        }
    }

    pub fn add_object(&mut self, o: impl Intersectable + 'static) {
        self.objects.push(Box::new(o));
    }

    pub fn add_light(&mut self, l: Light) {
        self.lights.push(l);
    }

    pub fn trace(&self, ray: Ray) -> Spectrum {
        let intersection = self.find_nearest_intersection(&ray);

        match intersection {
            None => spectrum::BLACK,
            Some(i) => self.lighting(i.point, i.normal, i.material),
        }
    }

    fn find_nearest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        assert!(!self.objects.is_empty());

        self.objects
            .iter()
            .flat_map(|x| x.intersect(ray))
            .filter(|x| !x.distance.is_nan())
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
    }

    fn diffuse_lighting(
        &self,
        point: Vector3<f64>,
        normal: Vector3<f64>,
        diffuse_color: Spectrum,
        light_pos: Vector3<f64>,
        light_power: Spectrum,
    ) -> Spectrum {
        let v = light_pos - point;
        let l = v.normalize();
        let dot = normal.dot(&l);

        if dot <= 0.0 {
            return spectrum::BLACK;
        }

        let r = v.len();
        let factor = dot / (4.0 * std::f64::consts::PI * r * r);
        light_power.scale(factor) * diffuse_color
    }

    fn lighting(&self, point: Vector3<f64>, normal: Vector3<f64>, material: Material) -> Spectrum {
        self.lights
            .iter()
            .map(|x| self.diffuse_lighting(point, normal, material.diffuse, x.pos, x.power))
            .fold(spectrum::BLACK, |a, b| a + b)
    }
}
