use crate::intersect::{Intersectable, Intersection};
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::spectrum::{self, Spectrum};
use crate::vector::Vector3;

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
            None => return spectrum::BLACK,
        };

        let material = intersection.material;
        let mut light = spectrum::BLACK;

        if intersection.normal.dot(&ray.dir) < 0.0 {
            // 鏡面反射成分
            if 0.0 < material.reflective {
                let reflection_ray = ray.dir.reflect(&intersection.normal);
                let color = self.trace(Ray::new(intersection.point, reflection_ray), depth + 1);
                light += color.scale(material.reflective) * material.diffuse;
            }

            // 屈折成分
            if 0.0 < material.refractive {
                let refraction = ray.dir.refract(
                    intersection.normal,
                    VACUUM_REFRACTIVE_INDEX / material.refractive_index,
                );

                let color = self.trace(Ray::new(intersection.point, refraction), depth + 1);
                light += color.scale(material.refractive) * material.diffuse;
            }

            // 拡散反射成分
            let kd = 1.0 - material.reflective;
            if 0.0 < kd {
                let color = self.lighting(intersection.point, intersection.normal, material);
                light += color.scale(kd);
            }
        } else {
            let refract = ray.dir.refract(
                -intersection.normal,
                material.refractive_index / VACUUM_REFRACTIVE_INDEX,
            );

            light = self.trace(Ray::new(intersection.point, refract), depth + 1);
        }

        light
    }

    fn find_nearest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        assert!(!self.objects.is_empty());

        self.objects
            .iter()
            .flat_map(|x| x.intersect(ray))
            .filter(|x| !x.distance.is_nan())
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
    }

    fn visible(&self, org: Vector3<f64>, target: Vector3<f64>) -> bool {
        let v = (target - org).normalize();
        let shadow_ray = Ray::new(org, v);

        self.objects
            .iter()
            .flat_map(|x| x.intersect(&shadow_ray))
            .all(|x| x.distance >= v.len())
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

        if dot > 0.0 && self.visible(point, light_pos) {
            let r = v.len();
            let factor = dot / (4.0 * std::f64::consts::PI * r * r);
            return light_power.scale(factor) * diffuse_color;
        }

        spectrum::BLACK
    }

    fn lighting(&self, point: Vector3<f64>, normal: Vector3<f64>, material: Material) -> Spectrum {
        self.lights
            .iter()
            .map(|x| self.diffuse_lighting(point, normal, material.diffuse, x.pos, x.power))
            .fold(spectrum::BLACK, |a, b| a + b)
    }
}
