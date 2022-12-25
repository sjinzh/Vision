extern crate nalgebra as na;
extern crate color_eyre;
extern crate vision;

use color_eyre::eyre::Result;

use std::fs;
use std::collections::HashMap;
use na::{Vector3,Matrix3};
use vision::io::olsen_loader::OlssenData;
use vision::sensors::camera::perspective::Perspective;
use vision::image::{features::{Match,ImageFeature}};
use vision::sfm::{triangulation::Triangulation,SFMConfig,bundle_adjustment::run_ba, epipolar::tensor::{BifocalType,EssentialDecomposition}, rotation_avg::{optimize_rotations_with_rcd_per_track,optimize_rotations_with_rcd}};
use vision::odometry::runtime_parameters::RuntimeParameters;
use vision::numerics::{loss, weighting};
use vision::{float,Float,load_runtime_conf};



fn main() -> Result<()> {
    color_eyre::install()?;
    let runtime_conf = load_runtime_conf();

    println!("--------");

    let data_ceiling_barcelona_path = format!("{}/Olsen/Ceiling_Barcelona/",runtime_conf.dataset_path);
    let data_set_door_path = format!("{}/Olsen/Door_Lund/",runtime_conf.dataset_path);
    let data_set_ahlströmer_path =  format!("{}/Olsen/Jonas_Ahlströmer/",runtime_conf.dataset_path);
    let data_set_fountain_path =  format!("{}/Olsen/fountain/",runtime_conf.dataset_path);
    let data_set_vasa_path =  format!("{}/Olsen/vasa_statue/",runtime_conf.dataset_path);
    let data_set_ninjo_path =  format!("{}/Olsen/nijo/",runtime_conf.dataset_path);
    let data_set_de_guerre_path =  format!("{}/Olsen/de_guerre/",runtime_conf.dataset_path);
    let data_set_fort_canning_path = format!("{}/Olsen/Fort_Channing_gate/",runtime_conf.dataset_path);
    let data_set_park_gate_path = format!("{}/Olsen/park_gate/",runtime_conf.dataset_path);
    let data_set_kronan_path = format!("{}/Olsen/kronan/",runtime_conf.dataset_path);
    let data_set_round_church_path = format!("{}/Olsen/round_church/",runtime_conf.dataset_path);
    
    let olsen_data_path = data_set_door_path;

    let feature_skip_count = 1;
    let olsen_data = OlssenData::new(&olsen_data_path);
    let positive_principal_distance = false;
    let invert_intrinsics = false; // they are already negative from decomp
    let normalize_features = false;
    let filter_tracks = true;

    let (cam_intrinsics_0,cam_extrinsics_0) = olsen_data.get_camera_intrinsics_extrinsics(0,positive_principal_distance);
    let (cam_intrinsics_1,cam_extrinsics_1) = olsen_data.get_camera_intrinsics_extrinsics(1,positive_principal_distance);
    let (cam_intrinsics_2,cam_extrinsics_2) = olsen_data.get_camera_intrinsics_extrinsics(2,positive_principal_distance);
    let (cam_intrinsics_3,cam_extrinsics_3) = olsen_data.get_camera_intrinsics_extrinsics(3,positive_principal_distance);
    let (cam_intrinsics_4,cam_extrinsics_4) = olsen_data.get_camera_intrinsics_extrinsics(4,positive_principal_distance);
    let (cam_intrinsics_5,cam_extrinsics_5) = olsen_data.get_camera_intrinsics_extrinsics(5,positive_principal_distance);
    let (cam_intrinsics_6,cam_extrinsics_6) = olsen_data.get_camera_intrinsics_extrinsics(6,positive_principal_distance);
    let (cam_intrinsics_7,cam_extrinsics_7) = olsen_data.get_camera_intrinsics_extrinsics(7,positive_principal_distance);
    let (cam_intrinsics_8,cam_extrinsics_8) = olsen_data.get_camera_intrinsics_extrinsics(8,positive_principal_distance);
    let (cam_intrinsics_9,cam_extrinsics_9) = olsen_data.get_camera_intrinsics_extrinsics(9,positive_principal_distance);

    let (cam_intrinsics_10,cam_extrinsics_10) = olsen_data.get_camera_intrinsics_extrinsics(10,positive_principal_distance);
    let (cam_intrinsics_11,cam_extrinsics_11) = olsen_data.get_camera_intrinsics_extrinsics(11,positive_principal_distance);
    // let (cam_intrinsics_12,cam_extrinsics_12) = olsen_data.get_camera_intrinsics_extrinsics(12,positive_principal_distance);
    // let (cam_intrinsics_13,cam_extrinsics_13) = olsen_data.get_camera_intrinsics_extrinsics(13,positive_principal_distance);
    // let (cam_intrinsics_14,cam_extrinsics_14) = olsen_data.get_camera_intrinsics_extrinsics(14,positive_principal_distance);

    // let (cam_intrinsics_17,cam_extrinsics_17) = olsen_data.get_camera_intrinsics_extrinsics(17,positive_principal_distance);
    // let (cam_intrinsics_17,cam_extrinsics_17) = olsen_data.get_camera_intrinsics_extrinsics(17,positive_principal_distance);
    // let (cam_intrinsics_18,cam_extrinsics_18) = olsen_data.get_camera_intrinsics_extrinsics(18,positive_principal_distance);
    // let (cam_intrinsics_19,cam_extrinsics_19) = olsen_data.get_camera_intrinsics_extrinsics(19,positive_principal_distance);
    // let (cam_intrinsics_20,cam_extrinsics_20) = olsen_data.get_camera_intrinsics_extrinsics(20,positive_principal_distance);
    // let (cam_intrinsics_21,cam_extrinsics_21) = olsen_data.get_camera_intrinsics_extrinsics(21,positive_principal_distance);
    // let (cam_intrinsics_22,cam_extrinsics_22) = olsen_data.get_camera_intrinsics_extrinsics(22,positive_principal_distance);
    // let (cam_intrinsics_23,cam_extrinsics_23) = olsen_data.get_camera_intrinsics_extrinsics(23,positive_principal_distance);
    // let (cam_intrinsics_24,cam_extrinsics_24) = olsen_data.get_camera_intrinsics_extrinsics(24,positive_principal_distance);

    // let (cam_intrinsics_23,cam_extrinsics_23) = olsen_data.get_camera_intrinsics_extrinsics(23,positive_principal_distance);
    // let (cam_intrinsics_24,cam_extrinsics_24) = olsen_data.get_camera_intrinsics_extrinsics(24,positive_principal_distance);



    let matches_4_1 = olsen_data.get_matches_between_images(4, 1);
    let matches_2_1 = olsen_data.get_matches_between_images(2, 1);
    let matches_3_2 = olsen_data.get_matches_between_images(3, 2);
    let matches_4_2 = olsen_data.get_matches_between_images(4, 2);
    let matches_4_3 = olsen_data.get_matches_between_images(4, 3);
    let matches_5_4 = olsen_data.get_matches_between_images(5, 4);
    let matches_5_6 = olsen_data.get_matches_between_images(5, 6);
    let matches_6_7 = olsen_data.get_matches_between_images(6, 7);
    let matches_7_8 = olsen_data.get_matches_between_images(7, 8);
    let matches_8_9 = olsen_data.get_matches_between_images(8, 9);

    let matches_9_10 = olsen_data.get_matches_between_images(9, 10);
    //let matches_10_9 = olsen_data.get_matches_between_images(10, 9);
    let matches_10_11 = olsen_data.get_matches_between_images(10, 11);
    // let matches_11_12 = olsen_data.get_matches_between_images(11, 12);
    // let matches_12_13 = olsen_data.get_matches_between_images(12, 13);
    // let matches_13_14 = olsen_data.get_matches_between_images(13, 14);

    // let matches_19_18 = olsen_data.get_matches_between_images(19, 18);
    // let matches_20_19 = olsen_data.get_matches_between_images(20, 19);
    // let matches_20_21 = olsen_data.get_matches_between_images(20, 21);
    // let matches_21_22 = olsen_data.get_matches_between_images(21, 22);
    // let matches_22_23 = olsen_data.get_matches_between_images(22, 23);
    // let matches_23_24 = olsen_data.get_matches_between_images(23, 24);
    // let matches_22_23 = olsen_data.get_matches_between_images(22, 23);
    // let matches_23_24 = olsen_data.get_matches_between_images(23, 24);

    let matches_4_1_subvec = matches_4_1.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_2_1_subvec = matches_2_1.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_3_2_subvec = matches_3_2.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_4_2_subvec = matches_4_2.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_4_3_subvec = matches_4_3.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_5_4_subvec = matches_5_4.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_5_6_subvec = matches_5_6.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_6_7_subvec = matches_6_7.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_7_8_subvec = matches_7_8.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_8_9_subvec = matches_8_9.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();

    // let matches_10_9_subvec = matches_10_9.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_9_10_subvec = matches_9_10.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    let matches_10_11_subvec = matches_10_11.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_11_12_subvec = matches_11_12.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_12_13_subvec = matches_12_13.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_13_14_subvec = matches_13_14.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();

    // let matches_18_17_subvec = matches_19_18.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_18_17_subvec = matches_19_18.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_19_18_subvec = matches_19_18.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_20_19_subvec = matches_20_19.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_20_21_subvec = matches_20_21.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_21_22_subvec = matches_21_22.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_22_23_subvec = matches_22_23.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_23_24_subvec = matches_23_24.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_22_23_subvec = matches_22_23.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();
    // let matches_23_24_subvec = matches_23_24.iter().enumerate().filter(|&(i,_)| i % feature_skip_count == 0).map(|(_,x)| x.clone()).collect::<Vec<Match<ImageFeature>>>();

    let pinhole_cam_0 = Perspective::from_matrix(&cam_intrinsics_0, invert_intrinsics);
    let pinhole_cam_1 = Perspective::from_matrix(&cam_intrinsics_1, invert_intrinsics);
    let pinhole_cam_2 = Perspective::from_matrix(&cam_intrinsics_2, invert_intrinsics);
    let pinhole_cam_3 = Perspective::from_matrix(&cam_intrinsics_3, invert_intrinsics);
    let pinhole_cam_4 = Perspective::from_matrix(&cam_intrinsics_4, invert_intrinsics);
    let pinhole_cam_5 = Perspective::from_matrix(&cam_intrinsics_5, invert_intrinsics);
    let pinhole_cam_6 = Perspective::from_matrix(&cam_intrinsics_6, invert_intrinsics);
    let pinhole_cam_7 = Perspective::from_matrix(&cam_intrinsics_7, invert_intrinsics);
    let pinhole_cam_8 = Perspective::from_matrix(&cam_intrinsics_8, invert_intrinsics);
    let pinhole_cam_9 = Perspective::from_matrix(&cam_intrinsics_9, invert_intrinsics);

    let pinhole_cam_10 = Perspective::from_matrix(&cam_intrinsics_10, invert_intrinsics);
    let pinhole_cam_11 = Perspective::from_matrix(&cam_intrinsics_11, invert_intrinsics);
    // let pinhole_cam_12 = Perspective::from_matrix(&cam_intrinsics_12, invert_intrinsics);
    // let pinhole_cam_13 = Perspective::from_matrix(&cam_intrinsics_13, invert_intrinsics);
    // let pinhole_cam_14 = Perspective::from_matrix(&cam_intrinsics_14, invert_intrinsics);
    // let pinhole_cam_17 = Perspective::from_matrix(&cam_intrinsics_17, invert_intrinsics);
    // let pinhole_cam_17 = Perspective::from_matrix(&cam_intrinsics_17, invert_intrinsics);
    // let pinhole_cam_18 = Perspective::from_matrix(&cam_intrinsics_18, invert_intrinsics);
    // let pinhole_cam_19 = Perspective::from_matrix(&cam_intrinsics_19, invert_intrinsics);
    // let pinhole_cam_20 = Perspective::from_matrix(&cam_intrinsics_20, invert_intrinsics);
    // let pinhole_cam_21 = Perspective::from_matrix(&cam_intrinsics_21, invert_intrinsics);
    // let pinhole_cam_22 = Perspective::from_matrix(&cam_intrinsics_22, invert_intrinsics);
    // let pinhole_cam_23 = Perspective::from_matrix(&cam_intrinsics_23, invert_intrinsics);
    // let pinhole_cam_24 = Perspective::from_matrix(&cam_intrinsics_24, invert_intrinsics);
    // let pinhole_cam_24 = Perspective::from_matrix(&cam_intrinsics_24, invert_intrinsics);



    // let sfm_all_matches = vec!(vec!(matches_5_4_subvec),vec!(matches_5_6_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6)]);
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>())]);
    // let paths = vec!(vec!(4),vec!(6));
    // let root_id = 5;

    // let sfm_all_matches = vec!(vec!(matches_5_4_subvec,matches_4_3_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (3, pinhole_cam_3)]);
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>())]);
    // let paths = vec!(vec!(4,3));
    // let root_id = 5;

    // let sfm_all_matches = vec!(vec!(matches_5_6_subvec,matches_6_7_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8)]);  
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>()),(2, pinhole_cam_2.cast::<f32>()),(7, pinhole_cam_7.cast::<f32>()),(8, pinhole_cam_8.cast::<f32>())]);  
    // let paths = vec!(vec!(6,7));
    // let root_id = 5;


    // let sfm_all_matches = vec!(vec!(matches_5_4_subvec),vec!(matches_5_6_subvec,matches_6_7_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8)]);  
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>()),(2, pinhole_cam_2.cast::<f32>()),(7, pinhole_cam_7.cast::<f32>()),(8, pinhole_cam_8.cast::<f32>())]);  
    // let paths = vec!(vec!(4),vec!(6,7));
    // let root_id = 5;

    let sfm_all_matches = vec!(vec!(matches_5_4_subvec, matches_4_3_subvec),vec!(matches_5_6_subvec, matches_6_7_subvec));
    let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8),(9, pinhole_cam_9)]);  
    let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>()),(2, pinhole_cam_2.cast::<f32>()),(7, pinhole_cam_7.cast::<f32>()),(8, pinhole_cam_8.cast::<f32>()),(9, pinhole_cam_9.cast::<f32>())]);  
    let paths = vec!(vec!(4,3),vec!(6,7));
    let root_id = 5;

    // let sfm_all_matches = vec!(vec!(matches_5_4_subvec, matches_4_3_subvec,matches_3_2_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8),(9, pinhole_cam_9)]);  
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>()),(2, pinhole_cam_2.cast::<f32>()),(7, pinhole_cam_7.cast::<f32>()),(8, pinhole_cam_8.cast::<f32>()),(9, pinhole_cam_9.cast::<f32>())]);  
    // let paths = vec!(vec!(4,3,2));
    // let root_id = 5;

    // let sfm_all_matches = vec!(vec!(matches_5_6_subvec, matches_6_7_subvec,matches_7_8_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8),(9, pinhole_cam_9)]);  
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>()),(2, pinhole_cam_2.cast::<f32>()),(7, pinhole_cam_7.cast::<f32>()),(8, pinhole_cam_8.cast::<f32>()),(9, pinhole_cam_9.cast::<f32>())]);  
    // let paths = vec!(vec!(6,7,8));
    // let root_id = 5;

    // let sfm_all_matches = vec!(vec!(matches_5_6_subvec, matches_6_7_subvec,matches_7_8_subvec,matches_8_9_subvec,matches_9_10_subvec,matches_10_11_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8),(9, pinhole_cam_9),(10, pinhole_cam_10),(11, pinhole_cam_11)]);  
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>()),(2, pinhole_cam_2.cast::<f32>()),(7, pinhole_cam_7.cast::<f32>()),(8, pinhole_cam_8.cast::<f32>()),(9, pinhole_cam_9.cast::<f32>()),(10, pinhole_cam_10.cast::<f32>()),(11, pinhole_cam_11.cast::<f32>())]);  
    // let paths = vec!(vec!(6,7,8,9,10,11));
    // let root_id = 5;


    // let sfm_all_matches = vec!(vec!(matches_5_4_subvec, matches_4_3_subvec,matches_3_2_subvec),vec!(matches_5_6_subvec, matches_6_7_subvec,matches_7_8_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8),(9, pinhole_cam_9)]);  
    // let camera_map_ba = HashMap::from([(5, pinhole_cam_5.cast::<f32>()), (4, pinhole_cam_4.cast::<f32>()), (6, pinhole_cam_6.cast::<f32>()), (3, pinhole_cam_3.cast::<f32>()),(2, pinhole_cam_2.cast::<f32>()),(7, pinhole_cam_7.cast::<f32>()),(8, pinhole_cam_8.cast::<f32>()),(9, pinhole_cam_9.cast::<f32>())]);  
    // let paths = vec!(vec!(4,3,2),vec!(6,7,8));
    // let root_id = 5;

    // let sfm_all_matches = vec!(vec!(matches_10_9_subvec),vec!(matches_10_11_subvec));
    // let camera_map = HashMap::from([(9, pinhole_cam_9),(10, pinhole_cam_10),(11, pinhole_cam_11)]);  
    // let camera_map_ba = HashMap::from([(9, pinhole_cam_9.cast::<f32>()),(10, pinhole_cam_10.cast::<f32>()),(11, pinhole_cam_11.cast::<f32>())]);  
    // let paths = vec!(vec!(9),vec!(11));
    // let root_id = 10;

    // let sfm_all_matches = vec!(vec!(matches_20_19_subvec,matches_19_18_subvec),vec!(matches_20_21_subvec, matches_21_22_subvec));
    // let camera_map = HashMap::from([(17, pinhole_cam_17),(17, pinhole_cam_17),(18, pinhole_cam_18),(19, pinhole_cam_19),(20, pinhole_cam_20),(21, pinhole_cam_21),(22, pinhole_cam_22),(23, pinhole_cam_23),(24, pinhole_cam_24),(23, pinhole_cam_23),(24, pinhole_cam_24)]);  
    // let camera_map_ba = HashMap::from([(17, pinhole_cam_17.cast::<f32>()),(17, pinhole_cam_17.cast::<f32>()),(18, pinhole_cam_18.cast::<f32>()),(19, pinhole_cam_19.cast::<f32>()),(20, pinhole_cam_20.cast::<f32>()),(21, pinhole_cam_21.cast::<f32>()),(22, pinhole_cam_22.cast::<f32>()),(23, pinhole_cam_23.cast::<f32>()),(24, pinhole_cam_24.cast::<f32>()),(23, pinhole_cam_23.cast::<f32>()),(24, pinhole_cam_24.cast::<f32>())]);  
    // let paths = vec!(vec!(19,18),vec!(21,22));
    // let root_id = 20;

    // let sfm_all_matches = vec!(vec!(matches_20_19_subvec),vec!(matches_20_21_subvec));
    // let camera_map = HashMap::from([(17, pinhole_cam_17),(18, pinhole_cam_18),(19, pinhole_cam_19),(20, pinhole_cam_20),(21, pinhole_cam_21),(22, pinhole_cam_22),(23, pinhole_cam_23),(24, pinhole_cam_24)]);  
    // let camera_map_ba = HashMap::from([(17, pinhole_cam_17.cast::<f32>()),(18, pinhole_cam_18.cast::<f32>()),(19, pinhole_cam_19.cast::<f32>()),(20, pinhole_cam_20.cast::<f32>()),(21, pinhole_cam_21.cast::<f32>()),(22, pinhole_cam_22.cast::<f32>()),(23, pinhole_cam_23.cast::<f32>()),(24, pinhole_cam_24.cast::<f32>())]);  
    // let paths = vec!(vec!(19),vec!(21));
    // let root_id = 20;

    // let sfm_all_matches = vec!(vec!(matches_20_19_subvec),vec!(matches_20_21_subvec));
    // let camera_map = HashMap::from([(17, pinhole_cam_17),(18, pinhole_cam_18),(19, pinhole_cam_19),(20, pinhole_cam_20),(21, pinhole_cam_21),(22, pinhole_cam_22),(23, pinhole_cam_23),(24, pinhole_cam_24)]);  
    // let camera_map_ba = HashMap::from([(17, pinhole_cam_17.cast::<f32>()),(18, pinhole_cam_18.cast::<f32>()),(19, pinhole_cam_19.cast::<f32>()),(20, pinhole_cam_20.cast::<f32>()),(21, pinhole_cam_21.cast::<f32>()),(22, pinhole_cam_22.cast::<f32>()),(23, pinhole_cam_23.cast::<f32>()),(24, pinhole_cam_24.cast::<f32>())]);  
    // let paths = vec!(vec!(19),vec!(21));
    // let root_id = 20;

    // let sfm_all_matches = vec!(vec!(matches_5_4_subvec, matches_4_3_subvec),vec!(matches_5_6_subvec, matches_6_7_subvec,matches_7_8_subvec));
    // let camera_map = HashMap::from([(5, pinhole_cam_5), (4, pinhole_cam_4), (6, pinhole_cam_6), (3, pinhole_cam_3),(2, pinhole_cam_2),(7, pinhole_cam_7),(8, pinhole_cam_8)]);  
    // let paths = vec!(    let positive_principal_distance = false;vec!(4),vec!(6,7));
    // let root_id = 5;

    let sfm_config_fundamental = SFMConfig::new(root_id, paths.clone(), camera_map, camera_map_ba, sfm_all_matches.clone(), BifocalType::FUNDAMENTAL, Triangulation::LINEAR, filter_tracks, 0.8, 0.3, true, olsen_data.width*olsen_data.height);
    let (initial_cam_motions_per_path,filtered_matches_per_path) = sfm_config_fundamental.compute_lists_from_maps();

    //This is only to satisfy current interface in ba
    let initial_cam_motions = initial_cam_motions_per_path.clone().into_iter().flatten().collect::<Vec<((usize,usize),(Vector3<Float>,Matrix3<Float>))>>();
    let initial_cam_poses = Some(initial_cam_motions);

    if initial_cam_poses.is_some(){
        for (_,(t,r)) in initial_cam_poses.as_ref().unwrap() {
            println!("t : {}",t);
            println!("r : {}",r);
            println!("-------");
        }
    }

    if filtered_matches_per_path.len() > 0 {

        let runtime_parameters = RuntimeParameters {
            pyramid_scale: 1.0,
            max_iterations: vec![1e5 as usize; 1],
            eps: vec![1e-6],
            step_sizes: vec![1e0],
            max_norm_eps: 1e-30, 
            delta_eps: 1e-30,
            taus: vec![1.0e0],
            lm: true,
            debug: true,
            show_octave_result: true,
            loss_function: Box::new(loss::TrivialLoss { eps: 1e-16, approximate_gauss_newton_matrices: false }), 
            intensity_weighting_function:  Box::new(weighting::SquaredWeight {}),
            //intensity_weighting_function:  Box::new(weighting::CauchyWeight {c: 0.01}),
            cg_threshold: 1e-6,
            cg_max_it: 2e3 as usize
        };

        let ((cam_positions,points),(s,debug_states_serialized)) = run_ba(&filtered_matches_per_path, &sfm_config_fundamental, Some(&initial_cam_motions_per_path), olsen_data.get_image_dim(), &runtime_parameters, 1.0);
        fs::write(format!("{}/olsen.txt",runtime_conf.local_data_path), s?).expect("Unable to write file");
        if runtime_parameters.debug {
            fs::write(format!("{}/olsen_debug.txt",runtime_conf.local_data_path), debug_states_serialized?).expect("Unable to write file");
        }
    }


    Ok(())
}

