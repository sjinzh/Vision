extern crate nalgebra as na;

use na::{Matrix3, Vector3, Isometry3};
use crate::image::features::{Match,Feature};
use crate::sensors::camera::Camera;
use crate::odometry::runtime_parameters::RuntimeParameters;
use crate::sfm::{SFMConfig,landmark::Landmark,bundle_adjustment::{camera_feature_map::CameraFeatureMap}};
use crate::Float;


pub mod camera_feature_map;
pub mod solver;
pub mod state;



pub fn run_ba<C : Camera + Copy, T : Feature>(all_matches: &Vec<Vec<Match<T>>>, sfm_config: &SFMConfig<C, T>, camera_data: &Vec<((usize, C),(usize,C))>, initial_cam_poses: &Option<Vec<(usize,(Vector3<Float>,Matrix3<Float>))>>,
                                img_dim : (usize,usize) ,runtime_parameters: &RuntimeParameters, pyramid_scale: Float, depth_prior: Float) 
                                -> ((Vec<Isometry3<Float>>, Vec<Vector3<Float>>), (serde_yaml::Result<String>, serde_yaml::Result<String>)){


    let (unique_camera_ids_sorted,unique_cameras_sorted_by_id) = sfm_config.compute_unqiue_ids_cameras_sorted();
    let path_id_pairs = sfm_config.compute_path_id_pairs();

    //TODO: transative motions
    let unique_camera_id_pairs = path_id_pairs.into_iter().flatten().collect();
    let mut feature_map = CameraFeatureMap::new(all_matches,unique_camera_ids_sorted, img_dim);
    feature_map.add_matches(&unique_camera_id_pairs,all_matches, pyramid_scale);

    //TODO: switch impl
    let mut state = feature_map.get_euclidean_landmark_state(initial_cam_poses.as_ref(), camera_data, depth_prior);
    //let mut state = feature_map.get_inverse_depth_landmark_state(Some(&initial_motion_decomp), depth_prior,&cameras);

    let observed_features = feature_map.get_observed_features(false);
    
    let some_debug_state_list = solver::optimize::<_,_,3>(&mut state, &unique_cameras_sorted_by_id, &observed_features, &runtime_parameters);
    let state_serialized = serde_yaml::to_string(&state.to_serial());
    let debug_states_serialized = serde_yaml::to_string(&some_debug_state_list);

    
    (state.as_matrix_point(), (state_serialized,debug_states_serialized))


}
