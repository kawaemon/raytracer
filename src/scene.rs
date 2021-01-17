use crate::intersect::{Intersectable, Intersection};
use crate::material::Material;
use crate::random;
use crate::ray::Ray;
use crate::spectrum::{self, Spectrum, BLACK};
use crate::vector::Vector3;

const RECURSION_LIMIT: u32 = 10000;
const VACUUM_REFRACTIVE_INDEX: f64 = 1.0;

pub struct Scene {
    objects: Vec<Box<dyn Intersectable + 'static>>,
    sky_color: Spectrum,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            sky_color: BLACK,
        }
    }

    pub fn add_object(&mut self, o: impl Intersectable + 'static) {
        self.objects.push(Box::new(o));
    }

    pub fn set_sky_color(&mut self, c: Spectrum) {
        self.sky_color = c;
    }

    fn interact_surface(
        &self,
        ray_dir: Vector3,
        point: Vector3,
        normal: Vector3,
        material: Material,
        eta: f64,
        depth: u32,
    ) -> Spectrum {
        let ks = material.reflective;
        let kt = material.refractive;

        let t = random(0.0, 1.0);

        if t < ks {
            // 鏡面反射
            let r = ray_dir.reflect(&normal);
            self.trace(Ray::new(point, r), depth + 1) * material.diffuse
        } else if t < ks + kt {
            // 屈折
            let r = ray_dir.refract(normal, eta);
            self.trace(Ray::new(point, r), depth + 1) * material.diffuse
        } else {
            let r = normal.random_hemisphere();
            let li = self.trace(Ray::new(point, r), depth + 1);

            let fr = material.diffuse.scale(1.0 / std::f64::consts::PI);
            let factor = 2.0 * std::f64::consts::PI * normal.dot(&r);
            (li * fr).scale(factor)
        }
    }

    pub fn trace(&self, ray: Ray, depth: u32) -> Spectrum {
        if RECURSION_LIMIT < depth {
            return spectrum::BLACK;
        }

        let intersection = match self.find_nearest_intersection(&ray) {
            Some(i) => i,
            None => return self.sky_color,
        };

        let m = intersection.material;
        let dot = intersection.normal.dot(&ray.dir);

        if dot < 0.0 {
            self.interact_surface(
                ray.dir,
                intersection.point,
                intersection.normal,
                m,
                VACUUM_REFRACTIVE_INDEX / m.refractive_index,
                depth,
            ) + m.emissive.scale(-dot)
        } else {
            self.interact_surface(
                ray.dir,
                intersection.point,
                -intersection.normal,
                m,
                m.refractive_index / VACUUM_REFRACTIVE_INDEX,
                depth,
            )
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
}
