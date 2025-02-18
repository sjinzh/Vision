extern crate nalgebra as na;
extern crate num_traits;
extern crate simba;

use std::hash::Hash;
use na::{Vector3, Isometry3,base::Scalar, RealField};
use simba::scalar::SupersetOf;
use num_traits::{float,NumAssign};
use crate::image::features::solver_feature::SolverFeature;
use crate::image::features::Feature;
use crate::sensors::camera::Camera;
use crate::odometry::runtime_parameters::RuntimeParameters;
use crate::sfm::{SFMConfig,compute_path_id_pairs,bundle_adjustment::{state_linearizer::StateLinearizer}};
use crate::Float;


pub mod state_linearizer;
pub mod solver;
pub mod state;

pub fn run_ba<F: serde::Serialize + float::Float + Scalar + NumAssign + RealField + SupersetOf<Float>, C : Camera<Float> + Copy, T : Feature + Clone + PartialEq + Eq + Hash + SolverFeature>(sfm_config: &SFMConfig<C, T>,img_dim : (usize,usize) ,runtime_parameters: &RuntimeParameters<F>) 
                                -> ((Vec<Isometry3<F>>, Vec<Vector3<F>>), (serde_yaml::Result<String>, serde_yaml::Result<String>)){


    let (unique_camera_ids_sorted,unique_cameras_sorted_by_id) = sfm_config.compute_unqiue_ids_cameras_root_first();
    let path_id_pairs = compute_path_id_pairs(sfm_config.root(), sfm_config.paths());

    let state_linearizer = StateLinearizer::new(unique_camera_ids_sorted);

    //TODO: switch impl on landmark state
    let (mut state, feature_location_lookup) = state_linearizer.get_euclidean_landmark_state(
        &path_id_pairs, 
        sfm_config.match_norm_map(), 
        sfm_config.abs_pose_map(), 
        sfm_config.abs_landmark_map(), 
        sfm_config.reprojection_error_map(),  
        sfm_config.unique_landmark_ids().len()
    );
    //let mut state = feature_map.get_inverse_depth_landmark_state(Some(&initial_motion_decomp), depth_prior,&cameras);
    
    let observed_features = state_linearizer.get_observed_features::<F>(&feature_location_lookup, sfm_config.unique_landmark_ids().len());
    
    let some_debug_state_list = solver::optimize::<_,_,_,3>(&mut state, &unique_cameras_sorted_by_id, &observed_features, runtime_parameters);
    let state_serialized = serde_yaml::to_string(&state.to_serial());
    let debug_states_serialized = serde_yaml::to_string(&some_debug_state_list);

    
    (state.as_matrix_point(), (state_serialized,debug_states_serialized))


}
