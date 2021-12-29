extern crate nalgebra as na;

use na::{DVector,DMatrix,Matrix, Dynamic, U4, VecStorage,Point3, Vector4};
use crate::{float,Float};
use crate::sensors::camera::Camera;
use crate::numerics::lie::left_jacobian_around_identity;
use crate::numerics::{max_norm, solver::{compute_cost,weight_jacobian_sparse,weight_residuals_sparse, calc_weight_vec, gauss_newton_step_with_schur}};
use crate::sfm::{landmark::Landmark,bundle_adjustment::{state::State, camera_feature_map::CameraFeatureMap}};
use crate::odometry::runtime_parameters::RuntimeParameters; //TODO remove dependency on odometry module

pub fn get_feature_index_in_residual(cam_id: usize, feature_id: usize, n_cams: usize) -> usize {
    2*(cam_id + feature_id*n_cams)
}

//TODO: inverse depth
/**
 * In the format [f1_cam1, f1_cam2,...]
 * Some entries may be 0 since not all cams see all points
 * */
pub fn get_estimated_features<C : Camera, L: Landmark<T> + Copy + Clone, const T: usize>(state: &State<L,T>, cameras: &Vec<C>,observed_features: &DVector<Float>, estimated_features: &mut DVector<Float>) -> () {
    let n_cams = state.n_cams;
    let n_points = state.n_points;
    assert_eq!(estimated_features.nrows(),2*n_points*n_cams);
    let mut position_world = Matrix::<Float,U4,Dynamic, VecStorage<Float,U4,Dynamic>>::from_element(n_points, 1.0);
    for j in 0..n_points {
        position_world.fixed_slice_mut::<3,1>(0,j).copy_from(&state.get_landmarks()[j].get_euclidean_representation().coords); 
    };
    for i in 0..n_cams {
        let cam_idx = 6*i;
        let pose = state.to_se3(cam_idx);
        let camera = &cameras[i];

        //TODO: use transform_into_other_camera_frame
        let transformed_points = pose*&position_world;
        for j in 0..n_points {
            let estimated_feature = camera.project(&transformed_points.fixed_slice::<3,1>(0,j));  
            
            let feat_id = get_feature_index_in_residual(i, j, n_cams);
            if !(observed_features[feat_id] == CameraFeatureMap::NO_FEATURE_FLAG && observed_features[feat_id+1] == CameraFeatureMap::NO_FEATURE_FLAG){
                estimated_features[feat_id] = estimated_feature.x;
                estimated_features[feat_id+1] = estimated_feature.y;
            }

        }

    }
}


pub fn compute_residual(estimated_features: &DVector<Float>, observed_features: &DVector<Float>, residual_vector: &mut DVector<Float>) -> () {
    assert_eq!(residual_vector.nrows(), estimated_features.nrows());
    for i in 0..residual_vector.nrows() {
        residual_vector[i] =  estimated_features[i] - observed_features[i];
    }
}

//TODO: inverse depth -> move pont def to landmark struct
pub fn compute_jacobian_wrt_object_points<C : Camera, L: Landmark<T> + Copy + Clone, const T: usize>(camera: &C, state: &State<L,T>, cam_idx: usize, point_idx: usize, i: usize, j: usize, jacobian: &mut DMatrix<Float>) -> (){
    let transformation = state.to_se3(cam_idx);
    let point = state.get_landmarks()[point_idx].get_euclidean_representation();
    let jacobian_world = state.jacobian_wrt_world_coordiantes(point_idx,cam_idx);
    let transformed_point = transformation*Vector4::<Float>::new(point[0],point[1],point[2],1.0);
    let projection_jacobian = camera.get_jacobian_with_respect_to_position_in_camera_frame(&transformed_point.fixed_rows::<3>(0));
    let local_jacobian = projection_jacobian*jacobian_world;

    jacobian.fixed_slice_mut::<2,T>(i,j).copy_from(&local_jacobian.fixed_slice::<2,T>(0,0));
}

pub fn compute_jacobian_wrt_camera_extrinsics<C : Camera, L: Landmark<T> + Copy + Clone, const T: usize>( camera: &C, state: &State<L,T>, cam_idx: usize, point: &Point3<Float> ,i: usize, j: usize, jacobian: &mut DMatrix<Float>) 
    -> (){
    let transformation = state.to_se3(cam_idx);
    let transformed_point = transformation*Vector4::<Float>::new(point[0],point[1],point[2],1.0);
    let lie_jacobian = left_jacobian_around_identity(&transformed_point.fixed_rows::<3>(0)); 

    let projection_jacobian = camera.get_jacobian_with_respect_to_position_in_camera_frame(&transformed_point.fixed_rows::<3>(0));
    let local_jacobian = projection_jacobian*lie_jacobian;

    jacobian.fixed_slice_mut::<2,6>(i,j).copy_from(&local_jacobian);
}

pub fn compute_jacobian<C : Camera, L: Landmark<T> + Copy + Clone, const T: usize>(state: &State<L,T>, cameras: &Vec<C>, jacobian: &mut DMatrix<Float>) -> () {
    //cam
    let number_of_cam_params = 6*state.n_cams;
    for cam_state_idx in (0..number_of_cam_params).step_by(6) {
        let cam_id = cam_state_idx/6;
        let camera = &cameras[cam_id];
        
        //landmark
        for point_id in 0..state.n_points {
            let point = state.get_landmarks()[point_id].get_euclidean_representation();

            let row = get_feature_index_in_residual(cam_id, point_id, state.n_cams);
            let a_j = cam_state_idx;
            let b_j = number_of_cam_params+(T*point_id);
            

            compute_jacobian_wrt_camera_extrinsics(camera , state, cam_state_idx,&point,row,a_j, jacobian);
            compute_jacobian_wrt_object_points(camera, state, cam_state_idx ,point_id,row,b_j, jacobian);

        }

    }

}

pub fn optimize<C : Camera,L: Landmark<T> + Copy + Clone, const T: usize>(state: &mut State<L,T>, cameras: &Vec<C>, observed_features: &DVector<Float>, runtime_parameters: &RuntimeParameters ) -> () {
    let mut new_state = state.clone();
    let state_size = 6*state.n_cams+T*state.n_points;
    let mut jacobian = DMatrix::<Float>::zeros(observed_features.nrows(),state_size);
    let mut residuals = DVector::<Float>::zeros(observed_features.nrows());
    let mut new_residuals = DVector::<Float>::zeros(observed_features.nrows());
    let mut estimated_features = DVector::<Float>::zeros(observed_features.nrows());
    let mut new_estimated_features = DVector::<Float>::zeros(observed_features.nrows());
    let mut weights_vec = DVector::<Float>::from_element(observed_features.nrows(),1.0);
    let mut target_arrowhead = DMatrix::<Float>::zeros(state_size, state_size);
    let mut g = DVector::<Float>::from_element(state_size,0.0); 
    let mut delta = DVector::<Float>::from_element(state_size,0.0); 

    get_estimated_features(state, cameras,observed_features, &mut estimated_features);
    compute_residual(&estimated_features, observed_features, &mut residuals);

    compute_jacobian(&state,&cameras,&mut jacobian);

    weight_residuals_sparse(&mut residuals, &weights_vec); 
    weight_jacobian_sparse(&mut jacobian, &weights_vec);


    let mut max_norm_delta = float::MAX;
    let mut delta_thresh = float::MIN;
    let mut delta_norm = float::MAX;
    let mut nu = 2.0;

    let mut mu: Option<Float> = match runtime_parameters.lm {
        true => None,
        false => Some(0.0)
    };
    let step = match runtime_parameters.lm {
        true => 1.0,
        false => runtime_parameters.step_sizes[0]
    };
    let tau = runtime_parameters.taus[0];
    let max_iterations = runtime_parameters.max_iterations[0];
    
    let mut cost = compute_cost(&residuals,&runtime_parameters.loss_function);
    let mut iteration_count = 0;
    while ((!runtime_parameters.lm && (cost.sqrt() > runtime_parameters.eps[0])) || (runtime_parameters.lm && (delta_norm > delta_thresh && max_norm_delta > runtime_parameters.max_norm_eps)))  && iteration_count < max_iterations  {
        if runtime_parameters.debug{
            println!("it: {}, avg_rmse: {}",iteration_count,cost.sqrt());
        }

        target_arrowhead.fill(0.0);
        g.fill(0.0);
        delta.fill(0.0);
        let (gain_ratio_denom, mu_val) 
            = gauss_newton_step_with_schur::<_,_,_,_,_,_,T, 6>(
                &mut target_arrowhead,
                &mut g,
                &mut delta,
                &residuals,
                &jacobian,
                mu,
                tau,
                state.n_cams,
                state.n_points
            ); 

        // let identity = DMatrix::<Float>::identity(state_size, state_size);
        // let (delta,g,gain_ratio_denom, mu_val) 
        //     = crate::numerics::solver::gauss_newton_step(&residuals,
        //          &(jacobian),
        //          &identity,
        //          mu,
        //          tau); 

        mu = Some(mu_val);
        let pertb = step*(&delta);
        new_state.update(&pertb);

        get_estimated_features(&new_state, cameras,observed_features, &mut new_estimated_features);
        compute_residual(&new_estimated_features, observed_features, &mut new_residuals);
        if runtime_parameters.weighting {
            calc_weight_vec(
                &new_residuals,
                &runtime_parameters.intensity_weighting_function,
                &mut weights_vec,
            );
        }
        weight_residuals_sparse(&mut new_residuals, &weights_vec);

        let new_cost = compute_cost(&new_residuals,&runtime_parameters.loss_function);
        let cost_diff = cost-new_cost;
        let gain_ratio = cost_diff/gain_ratio_denom;
        if runtime_parameters.debug {
            println!("cost: {}, new cost: {}, mu: {:?}, gain: {} , nu: {}",cost,new_cost, mu, gain_ratio, nu);
        }

        if gain_ratio > 0.0  || !runtime_parameters.lm {
            estimated_features.copy_from(&new_estimated_features);
            state.copy_from(&new_state); 

            cost = new_cost;

            max_norm_delta = max_norm(&g);
            delta_norm = pertb.norm(); 

            delta_thresh = runtime_parameters.delta_eps*(estimated_features.norm() + runtime_parameters.delta_eps);

            residuals.copy_from(&new_residuals);

            jacobian.fill(0.0);
            compute_jacobian(&state,&cameras,&mut jacobian);
            weight_jacobian_sparse(&mut jacobian, &weights_vec);

            let v: Float = 1.0 / 3.0;
            mu = Some(mu.unwrap() * v.max(1.0 - (2.0 * gain_ratio - 1.0).powi(3)));
            nu = 2.0;
        } else {
            new_state.copy_from(&state); 
            mu = Some(nu*mu.unwrap());
            nu *= 2.0;
        }

        iteration_count += 1;

        if mu.unwrap().is_infinite(){
            break;
        }

    }

    
}


