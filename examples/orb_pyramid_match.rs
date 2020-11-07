extern crate image as image_rs;
extern crate sift;

use std::path::Path;
use sift::pyramid::orb::{build_orb_pyramid,generate_feature_pyramid,generate_feature_descriptor_pyramid,  orb_runtime_parameters::OrbRuntimeParameters, generate_match_pyramid};
use sift::visualize::{visualize_pyramid_feature_with_orientation,display_matches_version_2};
use sift::matching::brief_descriptor::BriefDescriptor;
use sift::image::Image;

fn main() {
    let image_name = "lenna";
    let image_name_2 = "lenna_90";
    let image_format = "png";
    let image_folder = "images/";
    let image_out_folder = "output/";
    let image_path = format!("{}{}.{}",image_folder,image_name, image_format);
    let image_path_2 = format!("{}{}.{}",image_folder,image_name_2, image_format);
    let converted_file_out_path = format!("{}{}_features_with_descriptors.{}",image_out_folder,image_name,image_format);

    let gray_image = image_rs::open(&Path::new(&image_path)).unwrap().to_luma();
    let gray_image_2 = image_rs::open(&Path::new(&image_path_2)).unwrap().to_luma();

    //let mut display = Image::from_gray_image(&gray_image, false); 
    //let mut display_2 = Image::from_gray_image(&gray_image_2, false); 

    let runtime_params = OrbRuntimeParameters {
        min_image_dimensions: (50,50),
        sigma: 0.25,
        blur_radius: 5.0,
        octave_count: 3,
        harris_k: 0.04,
        fast_circle_radius: 3,
        fast_threshold_factor: 0.2,
        fast_consecutive_pixels: 12,
        fast_grid_size: (10,10),
        brief_n: 256,
        brief_s: 31
    };
    
    let pyramid = build_orb_pyramid(&gray_image, &runtime_params);
    let feature_pyramid = generate_feature_pyramid(&pyramid, &runtime_params);
    let feature_descriptor_pyramid_a = generate_feature_descriptor_pyramid(&pyramid,&feature_pyramid,&runtime_params);

    let pyramid_2 = build_orb_pyramid(&gray_image_2, &runtime_params);
    let feature_pyramid_2 = generate_feature_pyramid(&pyramid_2, &runtime_params);
    let feature_descriptor_pyramid_b = generate_feature_descriptor_pyramid(&pyramid_2,&feature_pyramid_2,&runtime_params);

    let match_pyramid = generate_match_pyramid(&feature_descriptor_pyramid_a,&feature_descriptor_pyramid_b);

    for i in 0..pyramid.octaves.len() {
        let display_a = &pyramid.octaves[i].images[0];
        let display_b = &pyramid_2.octaves[i].images[0];

        let matches = &match_pyramid.octaves[i];
        let match_dispay = display_matches_version_2(display_a, display_b, matches);
        let gray_image  = match_dispay.to_image();

        let name = format!("orb_match_{}",i);
        let file_path = format!("{}{}.{}",image_out_folder,name,image_format);
        gray_image.save(file_path).unwrap();
    }


    //let new_image = display.to_image();
    //new_image.save(converted_file_out_path).unwrap();

}