use crate::intersect::{Intersectable, Intersection};
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::spectrum::{self, Spectrum};
use crate::vector::Vector3;
use rand::{rngs::OsRng, Rng};

const RECURSION_LIMIT: u32 = 3000;
const VACUUM_REFRACTIVE_INDEX: f64 = 1.0;

pub struct Scene<'obj> {
    objects: Vec<Box<dyn Intersectable + 'obj>>,
    lights: Vec<Light>,
}

impl<'obj> Scene<'obj> {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            lights: vec![],
        }
    }

    pub fn add_object(&mut self, o: impl Intersectable + 'obj) {
        self.objects.push(Box::new(o));
    }

    pub fn add_light(&mut self, l: Light) {
        self.lights.push(l);
    }

    pub fn trace(&self, ray: Ray, depth: u32) -> Spectrum {
        if RECURSION_LIMIT < depth {
            println!("reached to recursion limit");
            return spectrum::BLACK;
        }

        let intersection = match self.find_nearest_intersection(&ray) {
            Some(i) => i,
            None => {
                return Spectrum {
                    r: 0.7,
                    g: 0.7,
                    b: 0.7,
                }
            }
        };

        let reflection_ray = intersection
            .normal
            .random_hemisphere(|| OsRng.gen_range(0.0, 1.0));
        let mut light = self.trace(
            Ray {
                origin: intersection.point,
                dir: reflection_ray,
            },
            depth + 1,
        );

        let fr = intersection
            .material
            .diffuse
            .scale(1.0 / std::f64::consts::PI);
        let factor = 2.0 * std::f64::consts::PI * intersection.normal.dot(&reflection_ray);

        (light * fr).scale(factor)
    }

    fn find_nearest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        assert!(!self.objects.is_empty());

        self.objects
            .iter()
            .flat_map(|x| x.intersect(ray))
            .filter(|x| !x.distance.is_nan())
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
    }

    fn visible(&self, org: Vector3, target: Vector3) -> bool {
        let v = (target - org).normalize();
        let shadow_ray = Ray::new(org, v);

        self.objects
            .iter()
            .flat_map(|x| x.intersect(&shadow_ray))
            .all(|x| x.distance >= v.len())
    }

    fn diffuse_lighting(
        &self,
        point: Vector3,
        normal: Vector3,
        diffuse_color: Spectrum,
        light_pos: Vector3,
        light_power: Spectrum,
    ) -> Spectrum {
        let v = light_pos - point;
        let l = v.normalize();
        let dot = normal.dot(&l);

        if dot > 0.0 && self.visible(point, light_pos) {
            let r = v.len();
            let factor = dot / (4.0 * std::f64::consts::PI * r * r);
            return light_power.scale(factor) * diffuse_color;
        }

        spectrum::BLACK
    }

    fn lighting(&self, point: Vector3, normal: Vector3, material: Material) -> Spectrum {
        self.lights
            .iter()
            .map(|x| self.diffuse_lighting(point, normal, material.diffuse, x.pos, x.power))
            .fold(spectrum::BLACK, |a, b| a + b)
    }
}
