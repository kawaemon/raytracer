use crate::spectrum::Spectrum;

#[derive(Clone)]
pub struct Material {
    pub diffuse: Spectrum,
    pub reflective: f64,
}
