use crate::spectrum::Spectrum;

#[derive(Clone)]
pub struct Material {
    pub diffuse: Spectrum,
    pub reflective: f64,
    pub refractive: f64,
    pub refractive_index: f64,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            diffuse: Spectrum::default(),
            reflective: 0.0,
            refractive: 0.0,
            refractive_index: 1.0,
        }
    }
}
