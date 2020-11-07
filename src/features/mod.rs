use crate::Float;

pub mod geometry;
pub mod sift_feature;
pub mod fast_feature;
pub mod octave_feature;
pub mod harris_corner;
pub mod hessian_response;
pub mod orb_feature;
pub mod intensity_centroid;



pub trait Feature {
    fn get_x_image(&self) -> usize;
    fn get_y_image(&self) -> usize;
    fn get_closest_sigma_level(&self) -> usize;
}

pub trait Oriented {
    fn get_orientation(&self) -> Float;
}