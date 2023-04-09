
extern crate nalgebra as na;
extern crate linear_ip;

use na::{MatrixXx3,DMatrix,DVector,Isometry3,Matrix4xX};
use std::collections::{HashMap,HashSet};
use std::hash::Hash;
use std::ops::AddAssign;
use crate::image::features::Feature;
use crate::Float;

/**
 * Outlier Rejection Using Duality Olsen et al.
 */
pub fn outlier_rejection_dual<Feat: Feature + Clone + Hash + PartialEq + Eq>(
        camera_ids_root_first: &Vec<usize>, 
        unique_landmark_ids: &mut HashSet<usize>, 
        abs_landmark_map: &mut HashMap<usize, Matrix4xX<Float>>,
        abs_pose_map: &mut HashMap<usize,Isometry3<Float>>, 
        feature_map: &mut HashMap<usize, HashSet<Feat>>,
        landmark_id_cam_pair_index_map: &HashMap<usize,Vec<((usize,usize),usize)>>,
        tol: Float
    ) -> DVector<Float> {
    assert_eq!(camera_ids_root_first.len(),abs_pose_map.keys().len());
    assert_eq!(camera_ids_root_first.len(),feature_map.keys().len());
    assert!(unique_landmark_ids.contains(&0)); // ids have to represent matrix indices

    let (a, b, c, a0, b0, c0) = generate_known_rotation_problem(unique_landmark_ids, camera_ids_root_first, abs_pose_map, feature_map);
    let (dual,slack) = solve_feasability_problem(a, b, c, a0, b0, c0, tol, 0.1, 100.0);
    update_maps(abs_pose_map, feature_map,unique_landmark_ids, dual);
    slack
}

#[allow(non_snake_case)]
fn solve_feasability_problem(a: DMatrix<Float>, b: DMatrix<Float>, c: DMatrix<Float>, a0: DVector<Float>, b0: DVector<Float>, c0: DVector<Float>, tol: Float, min_depth: Float, max_depth: Float) -> (DVector<Float>, DVector<Float>) {
    let (A,B,C, a_nrows, a1_ncols) = construct_feasability_inputs(a, b, c, a0, b0, c0, tol, min_depth, max_depth);
    let (_, Y, it, r_primal_norm, r_dual_norm) = linear_ip::solve(&A, &(-B), &C, 1e-8, 0.95, 0.1, 10, 1e-20); // goes OOM on wsl on large matricies
    println!("linear ip - it: {}, primal_norm: {}, dual_norm: {}",it, r_primal_norm, r_dual_norm);

    let mut s_temp = DVector::<Float>::zeros(Y.nrows()-a1_ncols);
    let s_temp_size = s_temp.nrows();
    assert_eq!(s_temp_size,6*a_nrows);

    let mut s = DVector::<Float>::zeros(a_nrows);
    s_temp.rows_mut(0, s_temp.nrows()).copy_from(&Y.rows(a1_ncols,s_temp_size));
    for offset in 0..6 {
        s.add_assign(s_temp.rows(offset*a_nrows,a_nrows));
    }

    let Y_new = Y.rows(0,a1_ncols).into_owned();

    (Y_new, s)
}

#[allow(non_snake_case)]
fn construct_feasability_inputs(a: DMatrix<Float>, b: DMatrix<Float>, c: DMatrix<Float>, a0: DVector<Float>, b0: DVector<Float>, c0: DVector<Float>, tol: Float, min_depth: Float, max_depth: Float) 
    -> (DMatrix<Float>, DVector<Float>, DVector<Float>, usize, usize) {
    let tol_c = tol*(&c);
    let tol_c0 = tol*(&c0);
    let mut a1 = DMatrix::<Float>::zeros(2*a.nrows() + 2*b.nrows(),a.ncols());
    a1.view_mut((0, 0),a.shape()).copy_from(&(-(&a)-&tol_c));
    a1.view_mut((a.nrows(), 0),a.shape()).copy_from(&((&a)-&tol_c));
    a1.view_mut((2*a.nrows(), 0),b.shape()).copy_from(&(-(&b)-&tol_c));
    a1.view_mut((2*a.nrows() + b.nrows(), 0),b.shape()).copy_from(&((&b)-&tol_c));

    let mut b1 = DVector::<Float>::zeros(2*a0.nrows() + 2*b0.nrows());
    b1.rows_mut(0, a0.nrows()).copy_from(&(&(a0)+&tol_c0));
    b1.rows_mut(a0.nrows(), a0.nrows()).copy_from(&(-(&a0)+&tol_c0));
    b1.rows_mut(2*a0.nrows(), b0.nrows()).copy_from(&((&b0)+&tol_c0));
    b1.rows_mut(2*a0.nrows() + b0.nrows(), b0.nrows()).copy_from(&(-(&b0)+&tol_c0));

    let mut a2 = DMatrix::<Float>::zeros(2*c.nrows(),c.ncols());
    a2.view_mut((0,0), c.shape()).copy_from(&-(&c));
    a2.view_mut((c.nrows(),0), c.shape()).copy_from(&c);

    let mut b2 = DVector::<Float>::zeros(2*c0.nrows());
    b2.rows_mut(0,c0.nrows()).copy_from(&(c0.add_scalar(-min_depth)));
    b2.rows_mut(c.nrows(),c0.nrows()).copy_from(&(-c0).add_scalar(max_depth));

    let mut A_temp = DMatrix::<Float>::zeros(a1.nrows()+a2.nrows(),a1.ncols());
    A_temp.view_mut((0,0),a1.shape()).copy_from(&a1);
    A_temp.view_mut((a1.nrows(),0),a2.shape()).copy_from(&a2);
    
    let mut A = DMatrix::<Float>::zeros(2*A_temp.nrows(),A_temp.ncols() + A_temp.nrows());
    let id = -DMatrix::<Float>::identity(A_temp.nrows(), A_temp.nrows());
    A.view_mut((0,0),A_temp.shape()).copy_from(&A_temp);
    A.view_mut((0,A_temp.ncols()),id.shape()).copy_from(&id);
    A.view_mut((A_temp.nrows(),0),A_temp.shape()).copy_from(&DMatrix::<Float>::zeros(A_temp.nrows(), A_temp.ncols()));
    A.view_mut((A_temp.nrows(),A_temp.ncols()),id.shape()).copy_from(&id);

    let mut C = DVector::<Float>::zeros(b1.nrows()+b2.nrows()+A_temp.nrows());
    C.rows_mut(0,b1.nrows()).copy_from(&b1);
    C.rows_mut(b1.nrows(),b2.nrows()).copy_from(&b2);

    let mut B = DVector::<Float>::zeros(a1.ncols()+A_temp.nrows());
    B.rows_mut(a1.ncols(),A_temp.nrows()).fill(1.0);

    (A.transpose(), B, C, a.nrows(), a1.ncols()) //TODO: transpose on construction
}

fn generate_known_rotation_problem<Feat: Feature + Clone>(unique_landmark_ids: &HashSet<usize>, camera_ids_root_first: &Vec<usize>, abs_pose_map: &mut HashMap<usize,Isometry3<Float>>, feature_map: &HashMap<usize, HashSet<Feat>>) -> (DMatrix<Float>,DMatrix<Float>,DMatrix<Float>,DVector<Float>,DVector<Float>,DVector<Float>) {
    let number_of_unique_points = unique_landmark_ids.len();
    let number_of_poses = abs_pose_map.len();
    let number_of_target_parameters = 3*number_of_unique_points + 3*(number_of_poses-1); // The first translation is taken as identity (origin) hence we dont optimize it
    let number_of_residuals = feature_map.values().fold(0, |acc, x| acc + x.len());

    let mut a = DMatrix::<Float>::zeros(number_of_residuals,number_of_target_parameters);
    let mut b = DMatrix::<Float>::zeros(number_of_residuals,number_of_target_parameters);
    let mut c = DMatrix::<Float>::zeros(number_of_residuals,number_of_target_parameters);

    let a0 = DVector::<Float>::zeros(number_of_residuals);
    let b0 = DVector::<Float>::zeros(number_of_residuals);
    let c0 = DVector::<Float>::zeros(number_of_residuals);

    let mut row_acc = 0;
    // Skip root cam since we assume origin (TOOD: Check this)
    for cam_idx in 1..number_of_poses {
        let cam_id = camera_ids_root_first[cam_idx];
        let rotation = abs_pose_map.get(&cam_id).expect("generate_known_rotation_problem: No rotation found").rotation.to_rotation_matrix();
        let rotation_matrix = rotation.matrix();
        let features = feature_map.get(&cam_id).expect("generate_known_rotation_problem: No features found");
        let number_of_points = features.len(); // asuming every feature's landmark id is distinct -> maybe make an explicit check?
        let ones = DVector::<Float>::repeat(number_of_points, 1.0);

        let mut p_data_x = DVector::<Float>::zeros(number_of_points);
        let mut p_data_y = DVector::<Float>::zeros(number_of_points);
        let mut p_col_ids = DVector::<usize>::zeros(number_of_points);
        // TODO: idx no longer represent the position in the sfm datastructures -> problem for update later
        for (idx, feature) in features.iter().enumerate() {
            p_data_x[idx] = feature.get_x_image_float();
            p_data_y[idx] = feature.get_y_image_float();
            p_col_ids[idx] = feature.get_landmark_id().expect("generate_known_rotation_problem: no landmark id"); 
        }
        let point_coeff_x = (&p_data_x)*rotation_matrix.row(2) - (&ones)*rotation_matrix.row(0);
        let point_coeff_y = (&p_data_y)*rotation_matrix.row(2) - (&ones)*rotation_matrix.row(1); 
        let point_coeff_z = (&ones)*rotation_matrix.row(2); 

        let mut t_coeff_x = MatrixXx3::<Float>::zeros(number_of_points);
        let mut t_coeff_y = MatrixXx3::<Float>::zeros(number_of_points);
        let mut t_coeff_z = MatrixXx3::<Float>::zeros(number_of_points);
        t_coeff_x.column_mut(0).copy_from(&(-(&ones)));
        t_coeff_y.column_mut(0).copy_from(&(-(&ones)));
        t_coeff_x.column_mut(2).copy_from(&p_data_x);
        t_coeff_y.column_mut(2).copy_from(&p_data_y);
        t_coeff_z.column_mut(2).copy_from(&ones);

        for p_idx in 0..number_of_points{
            let v_p_x = point_coeff_x.row(p_idx);
            let v_p_y = point_coeff_y.row(p_idx);
            let v_p_z = point_coeff_z.row(p_idx);
            let v_t_x = t_coeff_x.row(p_idx);
            let v_t_y = t_coeff_y.row(p_idx);
            let v_t_z = t_coeff_z.row(p_idx);
            let col_id = p_col_ids[p_idx];
            let r_idx = row_acc+p_idx;
            let column_l_idx = 3*col_id;
            let column_t_idx = 3*(number_of_unique_points+(cam_idx-1));
            a.fixed_view_mut::<1,3>(r_idx,column_l_idx).copy_from(&v_p_x);
            a.fixed_view_mut::<1,3>(r_idx,column_t_idx).copy_from(&v_t_x);
            b.fixed_view_mut::<1,3>(r_idx,column_l_idx).copy_from(&v_p_y);
            b.fixed_view_mut::<1,3>(r_idx,column_t_idx).copy_from(&v_t_y);
            c.fixed_view_mut::<1,3>(r_idx,column_l_idx).copy_from(&v_p_z);
            c.fixed_view_mut::<1,3>(r_idx,column_t_idx).copy_from(&v_t_z);
        }

        row_acc += number_of_points; 
    }

    (a,b,c,a0,b0,c0)
}

fn update_maps<Feat: Feature + Clone + Hash + PartialEq + Eq>(abs_pose_map: &mut HashMap<usize,Isometry3<Float>>, feature_map: &mut HashMap<usize, HashSet<Feat>>, unique_landmark_ids: &mut HashSet<usize>, dual: DVector<Float>) -> () {
    let num_cam = abs_pose_map.keys().len();
    let num_landmarks = unique_landmark_ids.len();
    let mut rejected_landmarks = HashSet::<usize>::with_capacity(num_landmarks*0.1 as usize);


    let landmark_view = dual.view((0,0),(num_landmarks,0));
    let translation_view = dual.view((num_landmarks,0),(dual.nrows(),0));

    for i in 0..num_landmarks {
        let landmark = landmark_view.fixed_rows::<3>(i);



    }

    //TODO
    // Update landmark positions
    // Update translation part of camera pose
    // generate rejected landmarks based on slack
    // remove landmarks from set
    

    for (_, features) in feature_map {
        let accepted_landmarks = features.drain().filter(|x| !rejected_landmarks.contains(&x.get_landmark_id().expect("update_maps: no landmark id found"))).collect::<Vec<Feat>>();
        assert!(features.is_empty());
        for accepted_l in accepted_landmarks {
                features.insert(accepted_l); 
        }
    }

    panic!("TODO: update_maps")
}