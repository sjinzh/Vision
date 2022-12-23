use nalgebra as na;
use nalgebra_sparse;

use nalgebra_sparse::{CooMatrix, CscMatrix};
use na::{Matrix3, MatrixXx3, Rotation3, Vector3};
use rand::{thread_rng, Rng};

use std::collections::HashMap;
use crate::{Float,float};
use crate::numerics::lie::angular_distance;


/**
    Rotation Coordiante Descent Parra et al.
 */
#[allow(non_snake_case)]
pub fn rcd(indexed_relative_rotations: &Vec<Vec<((usize, usize), Matrix3<Float>)>>, index_to_matrix_map :&HashMap<usize,usize>) -> MatrixXx3<Float> {
    let number_of_absolute_rotations = index_to_matrix_map.len();
    let relative_rotations_csc = generate_relative_rotation_matrix(&index_to_matrix_map,indexed_relative_rotations);
    let mut absolute_rotations = generate_absolute_rotation_matrix(number_of_absolute_rotations);
    let mut absolute_rotations_transpose = absolute_rotations.transpose();

    println!("{}",absolute_rotations);

    let mut cost = float::MAX;
    let mut old_cost = -0.5*(&absolute_rotations_transpose * (&relative_rotations_csc * &absolute_rotations)).trace();

    let max_epoch = 100; //TODO: config
    let eps = 1e-18;
    for _ in 0..max_epoch {
        for k in 0..number_of_absolute_rotations { 
            let W = generate_dense_from_csc_slice(k,number_of_absolute_rotations,&relative_rotations_csc);
            let BW = (&absolute_rotations)*(&absolute_rotations_transpose*&W);
            let A = &W.transpose()*&BW;
            let svd = A.svd(true,false);
            let S = svd.singular_values;
            let U =  svd.u.expect("Svd failed for rcd");
            let s = S.iter().map(|x| 1.0/x.sqrt()).collect::<Vec<_>>();
            let aux = BW*(U*Matrix3::<Float>::from_diagonal(&Vector3::<Float>::from_vec(s)))*U.transpose();

            let bottom_offset = 3*(number_of_absolute_rotations-k-1);
            absolute_rotations.rows_mut(0,3*k).copy_from(&aux.rows(0,3*k));
            absolute_rotations.slice_mut((3*k,0),(3,3)).copy_from(&Matrix3::<Float>::identity());
            absolute_rotations.rows_mut(absolute_rotations.nrows()-bottom_offset,bottom_offset).copy_from(&aux.rows(aux.nrows()-bottom_offset,bottom_offset));

            absolute_rotations_transpose.columns_mut(0,3*k).copy_from(&absolute_rotations.rows(0,3*k).transpose());
            absolute_rotations_transpose.slice_mut((0,3*k),(3,3)).copy_from(&Matrix3::<Float>::identity());
            absolute_rotations_transpose.columns_mut(absolute_rotations_transpose.ncols()-bottom_offset,bottom_offset).copy_from(&absolute_rotations.rows(absolute_rotations.nrows()-bottom_offset,bottom_offset).transpose())
        }

        cost = -0.5*(&absolute_rotations_transpose * (&relative_rotations_csc * &absolute_rotations)).trace();  
        println!("RCD cost: {}", cost);
        if (old_cost-cost) / old_cost.abs().max(1.0) <= eps {
            for i in 0..number_of_absolute_rotations {
                let mut Ri = absolute_rotations.fixed_rows::<3>(3*i).into_owned();
                if Ri.determinant() < 0.0 {
                    Ri *= -1.0;
                }
                absolute_rotations.fixed_rows_mut::<3>(3*i).copy_from(&Ri);
            }
            break;
        }
        old_cost = cost; 
    }
    
    println!("RCD completed with cost: {}", cost);
    absolute_rotations
}

pub fn optimize_rotations_with_rcd(indexed_relative_rotations: &Vec<Vec<((usize, usize), Matrix3<Float>)>>) -> Vec<Vec<((usize, usize), Matrix3<Float>)>> {
    let index_to_matrix_map = generate_path_indices_to_matrix_map(indexed_relative_rotations);
    //TODO: enforce direction! Also check if it just be run on each path individually!
    let absolute_rotations = rcd(indexed_relative_rotations, &index_to_matrix_map);
    println!("{}",absolute_rotations);
    absolute_to_relative_rotations(&absolute_rotations, indexed_relative_rotations, &index_to_matrix_map)
}

fn absolute_to_relative_rotations(absolute_rotations: &MatrixXx3<Float>, indexed_relative_rotations: &Vec<Vec<((usize, usize), Matrix3<Float>)>>, index_to_matrix_map: &HashMap<usize,usize>) -> Vec<Vec<((usize, usize), Matrix3<Float>)>>{
    indexed_relative_rotations.iter().map(|vec| {
        vec.iter().map(|((i_s, i_f), _)| {
            let idx_s = index_to_matrix_map.get(i_s).expect("RCD: Index s not present");
            let idx_f = index_to_matrix_map.get(i_f).expect("RCD: Index f not present");
            // Absolute rotations are already transposed!
            //((*i_s, *i_f),get_absolute_rotation_at(absolute_rotations,*idx_f).transpose()*get_absolute_rotation_at(absolute_rotations, *idx_s))
            ((*i_s, *i_f),(get_absolute_rotation_at(absolute_rotations,*idx_f).transpose()*get_absolute_rotation_at(absolute_rotations, *idx_s)).transpose()) // This works. check why last transpose is neccessary
            //(*i_s, *i_f),get_absolute_rotation_at(absolute_rotations,*idx_f)*get_absolute_rotation_at(absolute_rotations, *idx_s).transpose())
            //((*i_s, *i_f),get_absolute_rotation_at(absolute_rotations,*idx_f))
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>()
}

fn get_absolute_rotation_at(absolute_rotations: &MatrixXx3<Float>, index: usize) ->  Matrix3<Float> {
    absolute_rotations.fixed_rows::<3>(3*index).into_owned()
}

fn generate_path_indices_to_matrix_map(path_indices: &Vec<Vec<((usize, usize), Matrix3<Float>)>>) -> HashMap<usize,usize> {
    // assuming the first element of each vector is always the (same) root
    let number_of_rotations = path_indices.iter().map(|x| x.len()).sum::<usize>() + 1;
    let mut index_map = HashMap::<usize,usize>::with_capacity(number_of_rotations);
    let mut index_counter = 0;
    for v in path_indices {
        for ((i_s, i_f), _) in v {
            if !index_map.contains_key(i_s) {
                index_map.insert(*i_s, index_counter);
                index_counter += 1;
            }

            if !index_map.contains_key(i_f) {
                index_map.insert(*i_f, index_counter);
                index_counter += 1;
            }
        }
    }

    index_map
}

fn generate_relative_rotation_matrix(index_to_matrix_map: &HashMap<usize,usize>, indexed_relative_rotations: &Vec<Vec<((usize, usize), Matrix3<Float>)>>) -> CscMatrix<Float> {
    let number_of_views = index_to_matrix_map.len();
    let mut rotations_coo = CooMatrix::<Float>::zeros(3*number_of_views, 3*number_of_views);

    for v in indexed_relative_rotations {
        for ((i_s, i_f), rotation) in v {
            let idx_s = index_to_matrix_map.get(i_s).expect("RCD: Index s not present");
            let idx_f = index_to_matrix_map.get(i_f).expect("RCD: Index f not present");
            println!("rcd: Angular distance of {},{} is: {}",i_s,i_f,angular_distance(&rotation));
            let rotation_transpose = rotation.transpose();
            // Symmetric Matrix of transpose R_ij
            rotations_coo.push_matrix(3*idx_s, 3*idx_f, &rotation_transpose);
            rotations_coo.push_matrix(3*idx_f, 3*idx_s, &rotation_transpose);

        }
    }
    CscMatrix::from(&rotations_coo)
}

/**
 * This will be initialized with random rotations as stated in the paper
 */
fn generate_absolute_rotation_matrix(number_of_views: usize) -> MatrixXx3<Float> {
    let mut absolute_rotations = MatrixXx3::<Float>::zeros(3*number_of_views);
    let mut rng = thread_rng();

    for i in 0..number_of_views{
        let rot = rng.gen::<Rotation3<Float>>();
        absolute_rotations.fixed_rows_mut::<3>(3*i).copy_from(&rot.matrix());
    }
    absolute_rotations
}

/**
 * Creates a row slice for a single rotation starting at col_start
 */
pub fn generate_dense_from_csc_slice(rotation_index_start: usize, max_rotation_count: usize, relative_rotations_csc: &CscMatrix<Float>) -> MatrixXx3<Float> {
    let col_start = 3*rotation_index_start;
    let mut dense = MatrixXx3::<Float>::zeros(3*max_rotation_count);
    for col_offset in 0..3 {
        let current_col = col_start+col_offset;
        let col = relative_rotations_csc.col(current_col);
        for (row_index,v) in col.row_indices().iter().zip(col.values().iter()) {
            dense[(*row_index,col_offset)] = *v; 
        }
    }
    dense   
}