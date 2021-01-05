extern crate nalgebra as na;
extern crate image as image_rs;

use na::{Vector3, Quaternion};
use crate::image::Image;
use crate::camera::pinhole::Pinhole;
use crate::Float;


#[derive(Clone)]
pub struct LoadedData {
    pub source_gray_images: Vec<Image>,
    pub source_depth_images: Vec<Image>,
    pub target_gray_images: Vec<Image>,
    pub target_depth_images: Vec<Image>,
    pub pinhole_camera: Pinhole,
    pub target_gt_poses: Option<Vec<(Vector3<Float>,Quaternion<Float>)>>,
    pub source_gt_poses: Option<Vec<(Vector3<Float>,Quaternion<Float>)>> // Move to 
}