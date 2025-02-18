use crate::Float;
use crate::image::features::{Feature, Oriented, geometry::point::Point};

#[derive(Debug,Clone)]
pub struct KeyPoint {
    pub x: usize,
    pub y: usize,
    pub sigma_level: usize,
    pub octave_level: usize,
    pub orientation: Float
    //TODO: maybe put octave/orientation histogram here as well for debugging
}

impl Feature for KeyPoint {
    fn new(x: Float, y: Float, landmark_id: Option<usize>) -> KeyPoint { panic!("TODO: KeyPoint new") }
    fn get_location(&self) -> Point<Float> { Point::<Float> { x: self.get_x_image_float(), y: self.get_y_image_float() } }
    fn get_x_image_float(&self) -> Float { self.x as Float}
    fn get_y_image_float(&self) -> Float { self.y as Float}
    fn get_x_image(&self) -> usize {
        self.x
    }
    fn get_y_image(&self) -> usize {
        self.y
    }
    fn get_closest_sigma_level(&self) -> usize {
        self.sigma_level
    }
    fn apply_normalisation(&self, _: &nalgebra::Matrix3<Float>, _: Float) -> Self {
        panic!("TODO: KeyPoint apply_normalisation")
    }
    //TODO
    fn get_landmark_id(&self) -> Option<usize> {
        None
    }
    //TODO
    fn copy_with_landmark_id(&self, landmark_id: Option<usize>) -> Self {
        self.clone()
    }
}

impl Oriented for KeyPoint {
    fn get_orientation(&self) -> Float {
        self.orientation
    }
}
