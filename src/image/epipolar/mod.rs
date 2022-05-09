extern crate nalgebra as na;

mod five_point;

use na::{Vector3, Matrix3,Matrix,Dynamic,Rotation3, VecStorage, dimension::U9};
use crate::sensors::camera::Camera;
use crate::{Float,float};
use crate::image::features::{Feature,Match, ImageFeature, condition_matches};
pub type Fundamental =  Matrix3<Float>;
pub type Essential =  Matrix3<Float>;



#[derive(Clone,Copy)]
pub enum EssentialDecomposition {
    FÖRSNTER,
    KANATANI
} 

pub fn extract_matches<T: Feature>(matches: &Vec<Match<T>>, pyramid_scale: Float, normalize: bool) -> Vec<Match<ImageFeature>> {

    match normalize {
        true => {
            condition_matches(matches)
        },
        false => {
                matches.iter().map(|feature| {
                    let (r_x, r_y) = feature.feature_two.reconstruct_original_coordiantes_for_float(pyramid_scale);
                    let (l_x, l_y) = feature.feature_one.reconstruct_original_coordiantes_for_float(pyramid_scale);
                    Match { feature_one: ImageFeature::new(l_x,l_y), feature_two: ImageFeature::new(r_x,r_y)}
                }).collect()

        }
    }

}
#[allow(non_snake_case)]
pub fn five_point_essential<T: Feature + Clone, C: Camera>(matches: &[Match<T>; 5], camera_one: &C, camera_two: &C, depth_positive: bool) -> Essential {
    five_point::five_point_essential(matches,camera_one,camera_two,depth_positive)
}

/**
 * Photogrammetric Computer Vision p.570
 * Fails if points are coplanar!
 */
#[allow(non_snake_case)]
pub fn eight_point<T : Feature>(matches: &Vec<Match<T>>) -> Fundamental {
    let number_of_matches = matches.len() as Float; 
    assert!(number_of_matches >= 8.0);


    let mut A = Matrix::<Float,Dynamic, U9, VecStorage<Float,Dynamic,U9>>::zeros(matches.len());
    for i in 0..A.nrows() {
        let feature_right = matches[i].feature_two.get_as_2d_point();
        let feature_left = matches[i].feature_one.get_as_2d_point();

        let l_x =  feature_left[0];
        let l_y =  feature_left[1];

        let r_x =  feature_right[0];
        let r_y =  feature_right[1];

        A[(i,0)] = r_x*l_x;
        A[(i,1)] = r_x*l_y;
        A[(i,2)] = r_x;
        A[(i,3)] = r_y*l_x;
        A[(i,4)] = r_y*l_y;
        A[(i,5)] = r_y;
        A[(i,6)] = l_x;
        A[(i,7)] = l_y;
        A[(i,8)] = 1.0;
    }


    let svd = A.svd(false,true);
    let v_t =  &svd.v_t.expect("SVD failed on A");
    let f = &v_t.row(v_t.nrows()-1);
    let mut F = Matrix3::<Float>::zeros();
    F[(0,0)] = f[0];
    F[(1,0)] = f[1];
    F[(2,0)] = f[2];
    F[(0,1)] = f[3];
    F[(1,1)] = f[4];
    F[(2,1)] = f[5];
    F[(0,2)] = f[6];
    F[(1,2)] = f[7];
    F[(2,2)] = f[8];

    let mut svd_f = F.svd(true,true);
    let acc = svd_f.singular_values[0].powi(2) + svd_f.singular_values[1].powi(2);
    svd_f.singular_values[2] = 0.0;
    svd_f.singular_values /= acc.sqrt();
    svd_f.recompose().ok().expect("SVD recomposition failed")
}


//TODO: write a test for this
pub fn essential_matrix_from_motion(translation: &Vector3<Float>, rotation: &Matrix3<Float>) -> Matrix3<Float> {
    translation.cross_matrix()*rotation.transpose()
}

#[allow(non_snake_case)]
pub fn compute_essential(F: &Fundamental, projection_start: &Matrix3<Float>, projection_finish: &Matrix3<Float>) -> Essential {
    projection_start.transpose()*F*projection_finish
}

#[allow(non_snake_case)]
pub fn compute_fundamental(E: &Essential, inverse_projection_start: &Matrix3<Float>, inverse_projection_finish: &Matrix3<Float>) -> Essential {
    inverse_projection_start.transpose()*E*inverse_projection_finish
}

#[allow(non_snake_case)]
pub fn filter_matches_from_fundamental<T: Feature + Clone>(F: &Fundamental,matches: &Vec<Match<T>>, epipiolar_thresh: Float) -> Vec<Match<T>> {
    matches.iter().filter(|m| {
            let start = m.feature_one.get_as_2d_homogeneous();
            let finish = m.feature_two.get_as_2d_homogeneous();
            (start.transpose()*F*finish)[0].abs() < epipiolar_thresh
        }).cloned().collect::<Vec<Match<T>>>()
}

#[allow(non_snake_case)]
pub fn filter_matches_from_motion<T: Feature + Clone, C: Camera>(matches: &Vec<Match<T>>, relative_motion: &(Vector3<Float>,Matrix3<Float>),camera_pair: &(C,C),is_depth_positive: bool , epipiolar_thresh: Float) -> Vec<Match<T>> {
    let (cam_s,cam_f) = &camera_pair;
    let (t,R) = &relative_motion;
    let essential = essential_matrix_from_motion(t, R);
    let cam_s_inv = cam_s.get_inverse_projection();
    let cam_f_inv = cam_f.get_inverse_projection();
    let fundamental = compute_fundamental(&essential, &cam_s_inv, &cam_f_inv);

    filter_matches_from_fundamental(&fundamental,matches ,epipiolar_thresh)
}

/**
 * Photogrammetric Computer Vision p.583
 * @TODO: unify principal distance into enum
 */
#[allow(non_snake_case)]
pub fn decompose_essential_förstner<T : Feature>(
    E: &Essential, matches: &Vec<Match<T>>,
    inverse_camera_matrix_start: &Matrix3::<Float>,
    inverse_camera_matrix_finish: &Matrix3::<Float>, 
    is_depth_positive: bool) -> (Vector3<Float>, Matrix3<Float>,Matrix3<Float> ) {
    assert!(matches.len() > 0);
    let mut svd = E.svd(true,true);

    let u = &svd.u.expect("SVD failed on E");
    let v_t = &svd.v_t.expect("SVD failed on E");

    let W = Matrix3::<Float>::new(0.0, 1.0, 0.0,
                                 -1.0, 0.0 ,0.0,
                                  0.0, 0.0, 1.0);

    let Z = Matrix3::<Float>::new(0.0, 1.0, 0.0,
                                 -1.0, 0.0 ,0.0,
                                  0.0, 0.0, 0.0);

    let U_norm = u*u.determinant();
    let V = v_t.transpose();
    let V_norm = V*V.determinant();

    let e_corrected = U_norm* Matrix3::<Float>::new(1.0, 0.0, 0.0,
                                            0.0, 1.0 ,0.0,
                                            0.0, 0.0, 0.0)*V_norm.transpose();



    let Sb = u * Z * u.transpose();
    let b = Vector3::<Float>::new(Sb[(2, 1)],Sb[(0, 2)], Sb[(1,0)]);

    let R_matrices = vec!(V_norm*W*U_norm.transpose(),V_norm*W.transpose()*U_norm.transpose(), V_norm*W*U_norm.transpose(), V_norm*W.transpose()*U_norm.transpose());
    let h_vecs = vec!(b,b, -b, -b);

    let mut translation = Vector3::<Float>::zeros();
    let mut rotation = Matrix3::<Float>::identity();
    for i in 0..4 {
        let h = h_vecs[i];
        let R = R_matrices[i];
        let mut v_sign = 0.0;
        let mut u_sign = 0.0;
        for m in matches {
            let f_start = inverse_camera_matrix_start*m.feature_one.get_as_camera_ray();
            let f_finish = inverse_camera_matrix_finish*m.feature_two.get_as_camera_ray();

            let binormal = ((h.cross_matrix()*f_start).cross_matrix()*h).normalize();
            let mat = Matrix3::<Float>::from_columns(&[h,binormal,f_start.cross_matrix()*R.transpose()*f_finish]);
            let s_i = mat.determinant();
            let s_i_sign = match s_i {
                det if det > 0.0 => 1.0,
                det if det < 0.0 => -1.0,
                _ => 0.0
            };
            v_sign += s_i_sign;
            let s_r = (binormal.transpose()*R.transpose()*f_finish)[0];
            let s_r_sign = match s_r {
                s if s > 0.0 => 1.0,
                s if s < 0.0 => -1.0,
                _ => 0.0
            };
            u_sign += match s_i_sign*s_r_sign {
                s if s > 0.0 => 1.0,
                s if s < 0.0 => -1.0,
                _ => 0.0
            };
        }

        let u_sign_avg = u_sign /matches.len() as Float; 
        let v_sign_avg = v_sign /matches.len() as Float;

        if u_sign_avg > 0.0 && v_sign_avg > 0.0 {
            translation = h;
            rotation = R;
            break;
        }
    }

    if is_depth_positive {
        // translation was computed from the correct essential matrix so we dont have to change anything
        // due to alignment to negative depth, the rotation is actually form f_finish to f_start so we have to transpose.
        rotation = rotation.transpose();
    }

    (translation,rotation,e_corrected)

}

//TODO: this is still a little unclear depending on positive or negative depth
/**
 * Statistical Optimization for Geometric Computation p.338
 */
#[allow(non_snake_case)]
pub fn decompose_essential_kanatani<T: Feature>(E: &Essential, matches: &Vec<Match<T>>, is_depth_positive: bool) -> (Vector3<Float>, Matrix3<Float>, Matrix3<Float>) {
    assert!(matches.len() > 0);
    assert!(!is_depth_positive);
    println!("WARN: decompose_essential_kanatani is buggy");
    let svd = (E*E.transpose()).svd(true,false);
    let min_idx = svd.singular_values.imin();
    let u = &svd.u.expect("SVD failed on E");
    let mut h = u.column(min_idx).normalize();

    let sum_of_determinants = matches.iter().fold(0.0, |acc,m| {

        let (start_new,finish_new) = (m.feature_one.get_as_3d_point(-1.0),m.feature_one.get_as_3d_point(-1.0));

        let mat = Matrix3::from_columns(&[h,start_new,E*finish_new]);
        match mat.determinant() {
            v if v > 0.0 => acc+1.0,
            v if v < 0.0 => acc-1.0,
            _ => acc
        }
    });
    if sum_of_determinants < 0.0 {
        h  *= -1.0; 
    }

    let K = (-h).cross_matrix()*E;
    let mut svd_k = K.svd(true,true);
    let u_k = svd_k.u.expect("SVD U failed on K");
    let v_t_k = svd_k.v_t.expect("SVD V_t failed on K");
    let min_idx = svd_k.singular_values.imin();
    for i in 0..svd_k.singular_values.nrows(){
        if i == min_idx {
            svd_k.singular_values[i] = (u_k*v_t_k).determinant();
        } else {
            svd_k.singular_values[i] = 1.0;
        }
    }
    let R = svd_k.recompose().ok().expect("SVD recomposition failed on K");
    let translation = h;

    //TODO: corrected E
    (translation,R, Matrix3::<Float>::identity())

}

pub fn compute_initial_cam_motions<C : Camera + Copy,T : Feature + Clone>(all_matches: &Vec<Vec<Match<T>>>,camera_data: &Vec<((usize, C),(usize,C))>,pyramid_scale:Float, epipiolar_thresh: Float, is_depth_positive: bool, decomp_alg: EssentialDecomposition) 
    ->  Vec<(u64,(Vector3<Float>,Matrix3<Float>))> {
    let feature_machtes = all_matches.iter().filter(|m| m.len() >= 8).map(|m| extract_matches(m, pyramid_scale, true)).collect::<Vec<Vec<Match<ImageFeature>>>>();
    let fundamental_matrices = feature_machtes.iter().map(|m| eight_point(m)).collect::<Vec<Fundamental>>();
    let accepted_matches = fundamental_matrices.iter().zip(feature_machtes.iter()).map(|(f,m)| filter_matches_from_fundamental(f, m,epipiolar_thresh)).collect::<Vec<Vec<Match<ImageFeature>>>>();
    let essential_matrices_with_cameras = fundamental_matrices.iter().enumerate().map(|(i,f)| {
        let ((id1,c1),(id2,c2)) = camera_data[i];
        (id1,id2,compute_essential(f, &c1.get_projection(), &c2.get_projection()),c1,c2)
    }).collect::<Vec<(usize,usize,Essential, C, C)>>();

    let initial_motion_decomp = essential_matrices_with_cameras.iter().filter(|(id1,_,_,_,_)| *id1 == camera_data[0].0.0).enumerate().map(|(i,(_,id2,e,c1,c2))| {
        let matches = &accepted_matches[i];
        let (h,rotation,_) = match (decomp_alg,matches.len()) {
            (_,count) if count < 8 => (Vector3::<Float>::zeros(), Matrix3::<Float>::identity(),Matrix3::<Float>::identity()),
            (EssentialDecomposition::FÖRSNTER,_) => decompose_essential_förstner(e,matches,&c1.get_inverse_projection(),&c2.get_inverse_projection(), is_depth_positive),
            (EssentialDecomposition::KANATANI,_) => decompose_essential_kanatani(e,matches, is_depth_positive)
        };

        (*id2 as u64,(h,rotation))
    }).collect::<Vec<(u64,(Vector3<Float>,Matrix3<Float>))>>();

    initial_motion_decomp
}
