use crate::spectrum::{Spectrum, BLACK};

#[derive(Clone, Copy)]
pub struct Material {
    pub diffuse: Spectrum,
    pub reflective: f64,
    pub refractive: f64,
    pub refractive_index: f64,
    pub emissive: Spectrum,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            diffuse: Spectrum::default(),
            reflective: 0.0,
            refractive: 0.0,
            refractive_index: 1.0,
            emissive: BLACK,
        }
    }
}
