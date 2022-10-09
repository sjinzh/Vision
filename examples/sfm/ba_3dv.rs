extern crate image as image_rs;
extern crate vision;
extern crate color_eyre;
extern crate nalgebra as na;

use std::fs;
use std::collections::HashMap;
use color_eyre::eyre::Result;
use vision::sfm::{bundle_adjustment::run_ba, epipolar::tensor::{BifocalType,EssentialDecomposition}};
use vision::sensors::camera::pinhole::Pinhole;
use vision::odometry::runtime_parameters::RuntimeParameters;
use vision::numerics::{loss, weighting};
use vision::io::three_dv_loader;
use vision::sfm::SFMConfig;
use vision::{float,Float,load_runtime_conf};


fn main() -> Result<()> {

    color_eyre::install()?;
    let runtime_conf = load_runtime_conf();


    let matches_0_1 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_0.xyz", "image_formation_neg_z_no_noise_1.xyz");
    let matches_0_2 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_0.xyz", "image_formation_neg_z_no_noise_2.xyz");
    let matches_1_0 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_1.xyz", "image_formation_neg_z_no_noise_0.xyz");
    let matches_1_3 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_1.xyz", "image_formation_neg_z_no_noise_3.xyz");
    let matches_0_3 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_0.xyz", "image_formation_neg_z_no_noise_3.xyz");
    let matches_2_1 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_2.xyz", "image_formation_neg_z_no_noise_1.xyz");
    let matches_2_3 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_2.xyz", "image_formation_neg_z_no_noise_3.xyz");
    let matches_3_4 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_no_noise_3.xyz", "image_formation_neg_z_no_noise_4.xyz");

    let matches_0_1 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_0.xyz", "image_formation_neg_z_noise_1.xyz");
    let matches_0_2 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_0.xyz", "image_formation_neg_z_noise_2.xyz");
    let matches_1_0 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_1.xyz", "image_formation_neg_z_noise_0.xyz");
    let matches_1_3 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_1.xyz", "image_formation_neg_z_noise_3.xyz");
    let matches_0_3 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_0.xyz", "image_formation_neg_z_noise_3.xyz");
    let matches_2_1 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_2.xyz", "image_formation_neg_z_noise_1.xyz");
    let matches_2_3 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_2.xyz", "image_formation_neg_z_noise_3.xyz");
    let matches_3_4 = three_dv_loader::load_matches(&format!("{}/3dv",runtime_conf.dataset_path), "image_formation_neg_z_noise_3.xyz", "image_formation_neg_z_noise_4.xyz");


    let intensity_camera_0 = Pinhole::new(1000.0, 1000.0, 320.0, 240.0, true);
    let intensity_camera_1 = Pinhole::new(1000.0, 1000.0, 320.0, 240.0, true);
    let intensity_camera_2 = Pinhole::new(1000.0, 1000.0, 320.0, 240.0, true);
    let intensity_camera_3 = Pinhole::new(1000.0, 1000.0, 320.0, 240.0, true);
    let intensity_camera_4 = Pinhole::new(1000.0, 1000.0, 320.0, 240.0, true);


    let runtime_parameters = RuntimeParameters {
        pyramid_scale: 1.0,
        max_iterations: vec![1e5 as usize; 1],
        eps: vec![1e-3],
        step_sizes: vec![1e0],
        max_norm_eps: 1e-30, 
        delta_eps: 1e-30,
        taus: vec![1e0],
        lm: true,
        debug: true,

        show_octave_result: true,
        loss_function: Box::new(loss::TrivialLoss { eps: 1e-16, approximate_gauss_newton_matrices: false }), 
        intensity_weighting_function:  Box::new(weighting::SquaredWeight {}),
        cg_threshold: 1e-6,
      
        cg_max_it: 2000
    };

    let camera_map = HashMap::from([(0, intensity_camera_0), (1, intensity_camera_1),(2,intensity_camera_2),(3,intensity_camera_3),(4,intensity_camera_4)  ]);
    //let sfm_config = SFMConfig::new(2, vec!(vec!(1), vec!(3)), camera_map.clone(), camera_map, vec!(vec!(matches_2_1),vec!(matches_2_3)),
    //let sfm_config = SFMConfig::new(2, vec!(vec!(1,0), vec!(3)), camera_map.clone(), camera_map, vec!(vec!(matches_2_1,matches_1_0),vec!(matches_2_3)),
    //let sfm_config = SFMConfig::new(2, vec!(vec!(1), vec!(3,4)), camera_map.clone(), camera_map, vec!(vec!(matches_2_1),vec!(matches_2_3,matches_3_4)),
    let sfm_config = SFMConfig::new(2, vec!(vec!(1,0), vec!(3,4)), camera_map.clone(), camera_map, vec!(vec!(matches_2_1,matches_1_0),vec!(matches_2_3,matches_3_4)),
        BifocalType::ESSENTIAL, 320*240);

    //let epipolar_thresh = 4e-1;
    let perc_tresh = 1.0;
    let normalize_features = false;

    let (initial_cam_motions_per_path,filtered_matches_per_path) = sfm_config.compute_pairwise_cam_motions_with_filtered_matches(
        perc_tresh,
        normalize_features,
        sfm_config.epipolar_alg()
    );

    let ((cam_positions,points),(s,debug_states_serialized)) = run_ba(&filtered_matches_per_path, &sfm_config, Some(&initial_cam_motions_per_path), (480,640), &runtime_parameters, 1.0);
    //let ((cam_positions,points),(s,debug_states_serialized)) = run_ba(&sfm_config.matches(), &sfm_config, None, (480,640), &runtime_parameters, 1.0,depth_prior);
    fs::write(format!("{}/{}",runtime_conf.local_data_path,"3dv.txt"), s?).expect("Unable to write file");
    if runtime_parameters.debug {
        fs::write(format!("{}/{}",runtime_conf.local_data_path,"3dv_debug.txt"), debug_states_serialized?).expect("Unable to write file");
    }
   
    Ok(())

}