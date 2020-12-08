use crate::spectrum::Spectrum;

#[derive(Clone, Default)]
pub struct Material {
    pub diffuse: Spectrum,
    pub reflective: f64,
    pub refractive: f64,
    pub refractive_index: f64,
}
