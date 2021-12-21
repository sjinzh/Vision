extern crate nalgebra as na;

use crate::numerics::lie::exp_se3;
use crate::Float;
use na::{DVector, Matrix4, Vector3, Isometry3, Rotation3};

/**
 * This is ordered [cam_1,cam_2,..,cam_n,point_1,point_2,...,point_m]
 * cam is parameterized by [u_1,u_2,u_3,w_1,w_2,w_3]
 * point is parameterized by [x,y,z]
 * */
#[derive(Clone)]
pub struct State {
    pub data: DVector<Float>,
    pub n_cams: usize,
    pub n_points: usize,
}

impl State {


    //TODO make this configurable, jacobians should also be accessed from here
    pub const CAM_TRANSLATION_PARAM_SIZE: usize = 3;
    pub const CAM_ROTATION_PARAM_SIZE: usize = 3;
    pub const CAM_PARAM_SIZE: usize = State::CAM_TRANSLATION_PARAM_SIZE + State::CAM_ROTATION_PARAM_SIZE;
    pub const LANDMARK_PARAM_SIZE: usize = 3;

    pub fn update(&mut self, perturb: &DVector<Float>) -> () {

        for i in (0..State::CAM_PARAM_SIZE * self.n_cams).step_by(State::CAM_PARAM_SIZE) {
            let u = 1.0*perturb.fixed_rows::<{State::CAM_TRANSLATION_PARAM_SIZE}>(i);
            let w = 1.0*perturb.fixed_rows::<{State::CAM_ROTATION_PARAM_SIZE}>(i + State::CAM_TRANSLATION_PARAM_SIZE);
            let delta_transform = exp_se3(&u, &w);
            
            let current_transform = self.to_se3(i);

            let new_transform = delta_transform*current_transform;

            let new_translation = new_transform.fixed_slice::<{State::CAM_TRANSLATION_PARAM_SIZE},1>(0,State::CAM_TRANSLATION_PARAM_SIZE);
            self.data.fixed_slice_mut::<{State::CAM_TRANSLATION_PARAM_SIZE},1>(i,0).copy_from(&new_translation);

            let new_rotation = na::Rotation3::from_matrix(&new_transform.fixed_slice::<3,3>(0,0).into_owned());
            self.data.fixed_slice_mut::<{State::CAM_ROTATION_PARAM_SIZE},1>(i+State::CAM_TRANSLATION_PARAM_SIZE,0).copy_from(&(new_rotation.scaled_axis()));
        }

        for i in ((State::CAM_PARAM_SIZE * self.n_cams)..self.data.nrows()).step_by(State::LANDMARK_PARAM_SIZE) {
            self.data[i] += perturb[i]; 
            self.data[i + 1] += perturb[i + 1];
            self.data[i + 2] += perturb[i + 2];
        }
    }

    pub fn to_se3(&self, i: usize) -> Matrix4<Float> {
        assert!(i < self.n_cams*State::CAM_PARAM_SIZE);
        let translation = self.data.fixed_rows::<{State::CAM_TRANSLATION_PARAM_SIZE}>(i);
        let axis = na::Vector3::new(self.data[i+3],self.data[i+4],self.data[i+5]);
        let axis_angle = na::Rotation3::new(axis);
        let mut transform = Matrix4::<Float>::identity();
        transform.fixed_slice_mut::<3,3>(0,0).copy_from(axis_angle.matrix());
        transform.fixed_slice_mut::<{State::CAM_TRANSLATION_PARAM_SIZE},1>(0,State::CAM_TRANSLATION_PARAM_SIZE).copy_from(&translation);
        transform
    }

    pub fn as_matrix_point(&self) -> (Vec<Isometry3<Float>>, Vec<Vector3<Float>>) {
        let mut cam_positions = Vec::<Isometry3<Float>>::with_capacity(self.n_cams);
        let mut points = Vec::<Vector3<Float>>::with_capacity(self.n_points);

        for i in (0..State::CAM_PARAM_SIZE * self.n_cams).step_by(State::CAM_PARAM_SIZE) {
            let u = self.data.fixed_rows::<3>(i);
            let w = self.data.fixed_rows::<3>(i + 3);
            let se3 = exp_se3(&u, &w);
            let rotation = Rotation3::<Float>::from_matrix(&se3.fixed_slice::<3,3>(0,0).into_owned());
            cam_positions.push(Isometry3::<Float>::new(se3.fixed_slice::<3,1>(0,3).into_owned(),rotation.scaled_axis()));
        }

        for i in (State::CAM_PARAM_SIZE * self.n_cams..self.data.nrows()).step_by(3) {
            let point = self.data.fixed_rows::<3>(i);
            points.push(Vector3::from(point));
        }

        (cam_positions, points)
    }

    pub fn to_serial(&self) -> (Vec<[Float; 6]>, Vec<[Float; 3]>) {
        let mut cam_serial = Vec::<[Float; 6]>::with_capacity(self.n_cams);
        let mut points_serial = Vec::<[Float; 3]>::with_capacity(self.n_points);
        let number_of_cam_params = State::CAM_PARAM_SIZE * self.n_cams;

        for i in (0..number_of_cam_params).step_by(6) {
            let arr: [Float; 6] = [
                self.data[i],
                self.data[i + 1],
                self.data[i + 2],
                self.data[i + 3],
                self.data[i + 4],
                self.data[i + 5],
            ];
            cam_serial.push(arr);
        }

        for i in (number_of_cam_params..self.data.nrows()).step_by(3) {
            let arr: [Float; 3] = [self.data[i], self.data[i + 1], self.data[i + 2]];
            points_serial.push(arr);
        }

        (cam_serial, points_serial)
    }

    pub fn from_serial((cam_serial, points_serial): &(Vec<[Float; 6]>, Vec<[Float; 3]>)) -> State {
        let total_size = 6 * cam_serial.len() + 3 * points_serial.len();
        let mut data = DVector::<Float>::zeros(total_size);
        let cam_offset = 6 * cam_serial.len();

        for i in 0..cam_serial.len() {
            let arr = cam_serial[i];
            let offset = 6 * i;
            data[offset] = arr[0];
            data[offset + 1] = arr[1];
            data[offset + 2] = arr[2];
            data[offset + 3] = arr[3];
            data[offset + 4] = arr[4];
            data[offset + 5] = arr[5];
        }

        for i in 0..points_serial.len() {
            let arr = points_serial[i];
            let offset = cam_offset + i * 3;
            data[offset] = arr[0];
            data[offset + 1] = arr[1];
            data[offset + 2] = arr[2];
        }

        State {
            data,
            n_cams: cam_serial.len(),
            n_points: points_serial.len(),
        }
    }
}
