use crate::ray::Ray;
use crate::vector::Vector3;

#[derive(Default, Clone)]
pub struct Camera {
    eye: Vector3,
    origin: Vector3,
    xaxis: Vector3,
    yaxis: Vector3,
}

impl Camera {
    pub fn look_at(
        &mut self,
        eye: Vector3,
        target: Vector3,
        up: Vector3,
        fov: f64,
        width: u32,
        height: u32,
    ) {
        self.eye = eye;

        let v = (target - eye).normalize();
        self.xaxis = v.cross(up).normalize();
        self.yaxis = v.cross(self.xaxis);

        let image_plane = (height as f64 / 2.0) / (fov as f64 / 2.0).tan();
        let center = v.scale(image_plane);
        self.origin =
            center - self.xaxis.scale(0.5 * width as f64) - self.yaxis.scale(0.5 * height as f64);
    }

    pub fn ray(&self, x: f64, y: f64) -> Ray {
        let p = self.origin + self.xaxis.scale(x) + self.yaxis.scale(y);
        let dir = p.normalize();
        Ray::new(self.eye, dir)
    }
}
