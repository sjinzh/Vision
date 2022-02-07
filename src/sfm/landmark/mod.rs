extern crate nalgebra as na;

use na::{Isometry3, Point3, SVector, SMatrix};
use crate::Float;

pub mod euclidean_landmark;
pub mod inverse_depth_landmark;

pub trait Landmark<const T: usize> {
    const LANDMARK_PARAM_SIZE: usize = T;

    fn from_state(state: SVector<Float, T>) -> Self; 
    fn update(&mut self, perturb :&SVector<Float,T>) -> (); //TODO: change signature for inverse depth
    fn get_euclidean_representation(&self) -> Point3<Float>;
    fn transform_into_other_camera_frame(&self, other_cam_world: &Isometry3<Float>) -> Point3<Float>; //TODO: check this vs get_euclidean_representation
    fn jacobian(&self, world_to_cam: &Isometry3<Float>) -> SMatrix<Float,3,T>;
    fn get_state_as_vector(&self) -> &SVector<Float, T>;
    fn from_array(arr: &[Float; T]) -> Self;
    fn get_state_as_array(&self) -> [Float; T];
}