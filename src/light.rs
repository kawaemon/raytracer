use crate::spectrum::Spectrum;
use crate::vector::Vector3;

pub struct Light {
    pub pos: Vector3<f64>,
    pub power: Spectrum,
}
