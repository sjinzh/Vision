extern crate nalgebra as na;

use na::{U2,U4,U6,U9,SVector,DVector,Matrix4,Matrix,Dynamic,VecStorage};
use std::boxed::Box;

use crate::pyramid::gd::{GDPyramid,gd_octave::GDOctave};
use crate::sensors::camera::{pinhole::Pinhole};
use crate::odometry::runtime_parameters::RuntimeParameters;
use crate::odometry::visual_odometry::dense_direct::{RESIDUAL_DIM,ResidualDim,Identity, compute_full_jacobian,precompute_jacobians,compute_image_gradients, backproject_points, compute_residuals};
use crate::image::Image;
use crate::numerics::{lie,loss::LossFunction};
use crate::features::geometry::point::Point;
use crate::{Float,float};

pub fn run_trajectory(source_rgdb_pyramids: &Vec<GDPyramid<GDOctave>>,target_rgdb_pyramids: &Vec<GDPyramid<GDOctave>>, intensity_camera: &Pinhole, depth_camera: &Pinhole , runtime_parameters: &RuntimeParameters) -> Vec<Matrix4<Float>> {
    source_rgdb_pyramids.iter().zip(target_rgdb_pyramids.iter()).enumerate().map(|(i,(source,target))| run(i+1,&source,&target, intensity_camera,depth_camera,runtime_parameters)).collect::<Vec<Matrix4<Float>>>()
}

pub fn run(iteration: usize, source_rgdb_pyramid: &GDPyramid<GDOctave>,target_rgdb_pyramid: &GDPyramid<GDOctave>, intensity_camera: &Pinhole,depth_camera: &Pinhole, runtime_parameters: &RuntimeParameters) -> Matrix4<Float> {
    let octave_count = source_rgdb_pyramid.octaves.len();

    assert_eq!(octave_count,runtime_parameters.taus.len());
    assert_eq!(octave_count,runtime_parameters.step_sizes.len());
    assert_eq!(octave_count,runtime_parameters.max_iterations.len());

    let depth_image = &source_rgdb_pyramid.depth_image;
    let mut lie_result = SVector::<Float,RESIDUAL_DIM>::zeros();
    let mut mat_result = Matrix4::<Float>::identity();
    
    for index in (0..octave_count).rev() {

        let result = estimate(&source_rgdb_pyramid.octaves[index],depth_image,&target_rgdb_pyramid.octaves[index],index,&lie_result,&mat_result,intensity_camera,depth_camera,runtime_parameters);
        lie_result = result.0;
        mat_result = result.1;
        let solver_iterations = result.2;

        if runtime_parameters.show_octave_result {
            println!("{}, est_transform: {}, solver iterations: {}",iteration,mat_result, solver_iterations);
        }

    }

    if runtime_parameters.show_octave_result {
        println!("final: est_transform: {}",mat_result);
    }

    mat_result
}

//TODO: buffer all debug strings and print at the end. Also the numeric matricies could be buffered per octave level
fn estimate(source_octave: &GDOctave, source_depth_image_original: &Image, target_octave: &GDOctave, octave_index: usize, initial_guess_lie: &SVector<Float,RESIDUAL_DIM>,initial_guess_mat: &Matrix4<Float>, intensity_camera: &Pinhole, depth_camera: &Pinhole, runtime_parameters: &RuntimeParameters) -> (SVector<Float,RESIDUAL_DIM>,Matrix4<Float>, usize) {
    let source_image = &source_octave.gray_images[0];
    let target_image = &target_octave.gray_images[0];
    let x_gradient_image = &target_octave.x_gradients[0];
    let y_gradient_image = &target_octave.y_gradients[0];
    let (rows,cols) = source_image.buffer.shape();
    let number_of_pixels = rows*cols;
    let number_of_pixels_float = number_of_pixels as Float;
    let mut percentage_of_valid_pixels = 100.0;

    let mut weights_vec = DVector::<Float>::from_element(number_of_pixels,1.0);
    let identity = Identity::identity();
    let mut residuals = DVector::<Float>::from_element(number_of_pixels,float::MAX);
    let mut residuals_unweighted = DVector::<Float>::from_element(number_of_pixels,float::MAX);
    let mut new_residuals_unweighted = DVector::<Float>::from_element(number_of_pixels,float::MAX);
    let mut new_residuals = DVector::<Float>::from_element(number_of_pixels,float::MAX);
    let mut full_jacobian = Matrix::<Float,Dynamic,ResidualDim, VecStorage<Float,Dynamic,ResidualDim>>::zeros(number_of_pixels); 
    let mut image_gradients =  Matrix::<Float,Dynamic,U2, VecStorage<Float,Dynamic,U2>>::zeros(number_of_pixels);
    let mut image_gradient_points = Vec::<Point<usize>>::with_capacity(number_of_pixels);
    let mut new_image_gradient_points = Vec::<Point<usize>>::with_capacity(number_of_pixels);
    let mut rescaled_jacobian_target = Matrix::<Float,Dynamic,ResidualDim, VecStorage<Float,Dynamic,ResidualDim>>::zeros(number_of_pixels); 
    let mut rescaled_residual_target =  DVector::<Float>::zeros(number_of_pixels);
    let (backprojected_points,backprojected_points_flags) = backproject_points(&source_image.buffer, &source_depth_image_original.buffer, &depth_camera, octave_index);
    let constant_jacobians = precompute_jacobians(&backprojected_points,&backprojected_points_flags,&intensity_camera);

    let mut est_transform = initial_guess_mat.clone();
    let mut est_lie = initial_guess_lie.clone();

    compute_residuals(&target_image.buffer, &source_image.buffer, &backprojected_points,&backprojected_points_flags,&est_transform, &intensity_camera, &mut residuals,&mut image_gradient_points);
    residuals_unweighted.copy_from(&residuals);
    weight_residuals_sparse(&mut residuals, &weights_vec);
    let mut cost = compute_cost(&residuals,&runtime_parameters.loss_function);

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
        false => runtime_parameters.step_sizes[octave_index]
    };
    let tau = runtime_parameters.taus[octave_index];
    let max_iterations = runtime_parameters.max_iterations[octave_index];
    

    compute_image_gradients(&x_gradient_image.buffer,&y_gradient_image.buffer,&image_gradient_points,&mut image_gradients);
    compute_full_jacobian(&image_gradients,&constant_jacobians,&mut full_jacobian);


    weight_jacobian_sparse(&mut full_jacobian, &weights_vec);
    

    let mut iteration_count = 0;
    while ((!runtime_parameters.lm && (cost.sqrt() > runtime_parameters.eps[octave_index])) || (runtime_parameters.lm && (delta_norm >= delta_thresh && max_norm_delta > runtime_parameters.max_norm_eps)))  && iteration_count < max_iterations  {
        if runtime_parameters.debug{
            println!("it: {}, rmse: {}, valid pixels: {}%",iteration_count,cost.sqrt(),percentage_of_valid_pixels);
        }

        let (delta,g,gain_ratio_denom, mu_val) 
            = gauss_newton_step_with_loss(&residuals,&residuals_unweighted, &full_jacobian, &identity, mu, tau, cost, & runtime_parameters.loss_function, &mut rescaled_jacobian_target,&mut rescaled_residual_target);
        mu = Some(mu_val);

        let pertb = step*delta;
        let new_est_transform = est_transform*lie::exp(&pertb.fixed_slice::<3, 1>(0, 0),&pertb.fixed_slice::<3, 1>(3, 0));

        new_image_gradient_points.clear();
        compute_residuals(&target_image.buffer, &source_image.buffer, &backprojected_points,&backprojected_points_flags,&new_est_transform, &intensity_camera , &mut new_residuals,&mut new_image_gradient_points);
        new_residuals_unweighted.copy_from(&new_residuals);
        
        
        if runtime_parameters.weighting {
            //dense_direct::compute_t_dist_weights(&new_residuals,&mut weights_vec,new_image_gradient_points.len() as Float,5.0,20,1e-10);
            norm(&new_residuals,&runtime_parameters.loss_function,&mut weights_vec);
        }
        weight_residuals_sparse(&mut new_residuals, &weights_vec);

        percentage_of_valid_pixels = (new_image_gradient_points.len() as Float/number_of_pixels_float) *100.0;
        let new_cost = compute_cost(&new_residuals,&runtime_parameters.loss_function);
        let cost_diff = cost-new_cost;
        let gain_ratio = cost_diff/gain_ratio_denom;
        if runtime_parameters.debug {
            println!("{},{}",cost,new_cost);
        }
        if gain_ratio >= 0.0  || !runtime_parameters.lm {
            est_lie.fixed_rows_mut::<6>(0).copy_from(&lie::ln(&new_est_transform));
            est_transform = new_est_transform.clone();
            cost = new_cost;

            max_norm_delta = g.max();
            delta_norm = delta.norm();
            delta_thresh = runtime_parameters.delta_eps*(est_lie.norm() + runtime_parameters.delta_eps);

            image_gradient_points = new_image_gradient_points.clone();
            residuals.copy_from(&new_residuals);
            residuals_unweighted.copy_from(&new_residuals_unweighted);

            compute_image_gradients(&x_gradient_image.buffer,&y_gradient_image.buffer,&image_gradient_points,&mut image_gradients);
            compute_full_jacobian(&image_gradients,&constant_jacobians,&mut full_jacobian);
            weight_jacobian_sparse(&mut full_jacobian, &weights_vec);

            let v: Float = 1.0/3.0;
            mu = Some(mu.unwrap() * v.max(1.0-(2.0*gain_ratio-1.0).powi(3)));
            nu = 2.0;
        } else {

            mu = Some(nu*mu.unwrap());
            nu *= 2.0;
        }

        iteration_count += 1;

    }

    (est_lie,est_transform,iteration_count)
}

fn norm(residuals: &DVector<Float>,loss_function :&Box<dyn LossFunction>, weights_vec: &mut DVector<Float>) ->() {
    for i in 0..residuals.len() {
        let res = residuals[i];
        weights_vec[i] = (loss_function.second_derivative_at_current(res) * res).abs().sqrt();
    }

}



fn weight_residuals_sparse(residual_target: &mut DVector<Float>, weights_vec: &DVector<Float>) -> () {
    residual_target.component_mul_assign(weights_vec);
}

//TODO: optimize
fn weight_jacobian_sparse(jacobian: &mut Matrix<Float,Dynamic,ResidualDim,VecStorage<Float,Dynamic,ResidualDim>>, weights_vec: &DVector<Float>) -> () {
    let size = weights_vec.len();
    for i in 0..size{
        let weighted_row = jacobian.row(i)*weights_vec[i];
        jacobian.row_mut(i).copy_from(&weighted_row);
    }
}

//TODO: potential for optimization. Maybe use less memory/matrices. 
#[allow(non_snake_case)]
fn gauss_newton_step_with_loss(
    residuals: &DVector<Float>, 
    residuals_unweighted: &DVector<Float>, 
    jacobian: &Matrix<Float,Dynamic,ResidualDim,VecStorage<Float,Dynamic,ResidualDim>>,
    identity: &Identity,
     mu: Option<Float>, 
     tau: Float, 
     current_cost: Float, 
     loss_function: &Box<dyn LossFunction>,
      rescaled_jacobian_target: &mut Matrix<Float,Dynamic,ResidualDim,VecStorage<Float,Dynamic,ResidualDim>>, 
      rescaled_residuals_target: &mut DVector<Float>) -> (SVector<Float,RESIDUAL_DIM>,SVector<Float,RESIDUAL_DIM>,Float,Float) {

    let selected_root = loss_function.select_root(current_cost);
    let first_deriv_at_cost = loss_function.first_derivative_at_current(current_cost);
    let second_deriv_at_cost = loss_function.second_derivative_at_current(current_cost);
    let is_curvature_negative = second_deriv_at_cost*current_cost < -0.5*first_deriv_at_cost;
    let (A,g) =  match selected_root {
        root if root != 0.0 => {
            match is_curvature_negative {
                false => {
                    let first_derivative_sqrt = loss_function.first_derivative_at_current(current_cost).sqrt();
                    let jacobian_factor = selected_root/current_cost;
                    let residual_scale = first_derivative_sqrt/(1.0-selected_root);
                    let res_j = residuals.transpose()*jacobian;
                    for i in 0..jacobian.nrows(){
                        rescaled_jacobian_target.row_mut(i).copy_from(&(first_derivative_sqrt*(jacobian.row(i) - (jacobian_factor*residuals[i]*res_j))));
                        rescaled_residuals_target[i] = residual_scale*residuals[i];
                    }
                    (rescaled_jacobian_target.transpose()*rescaled_jacobian_target as &Matrix<Float,Dynamic,ResidualDim,VecStorage<Float,Dynamic,ResidualDim>>,rescaled_jacobian_target.transpose()*rescaled_residuals_target as &DVector<Float>)
                },
                _ => (jacobian.transpose()*(first_deriv_at_cost*identity + 2.0*second_deriv_at_cost*residuals_unweighted*residuals_unweighted.transpose())*jacobian,first_deriv_at_cost*jacobian.transpose()*residuals) 
            }   
        },
        _ => (jacobian.transpose()*jacobian,jacobian.transpose()*residuals)
    };
    let mu_val = match mu {
        None => tau*A.diagonal().max(),
        Some(v) => v
    };

    let decomp = (A+ mu_val*identity).qr();
    let h = decomp.solve(&(-g)).expect("QR Solve Failed");
    let gain_ratio_denom = h.transpose()*(mu_val*h-g);
    (h,g,gain_ratio_denom[0], mu_val)
}


fn compute_cost(residuals: &DVector<Float>, loss_function: &Box<dyn LossFunction>) -> Float {
    loss_function.cost((residuals.transpose()*residuals)[0])
}
