use crate::image::Image;
use crate::pyramid::Pyramid;
use self::{orb_octave::OrbOctave, orb_runtime_parameters::OrbRuntimeParameters};
use crate::features::{geometry::point::Point,orb_feature::OrbFeature};
use crate::matching::brief_descriptor::BriefDescriptor;
use crate::Float;


pub mod orb_octave;
pub mod orb_runtime_parameters;

pub fn build_orb_pyramid(base_gray_image: Image, runtime_parameters: &OrbRuntimeParameters) -> Pyramid<OrbOctave> {

    let mut octaves: Vec<OrbOctave> = Vec::with_capacity(runtime_parameters.octave_count);

    let mut octave_image = base_gray_image;
    let mut sigma = runtime_parameters.sigma;

    for i in 0..runtime_parameters.octave_count {

        if i > 0 {
            octave_image = Image::downsample_half(&octaves[i-1].images[0], false,  runtime_parameters.min_image_dimensions);
            sigma *= 2.0;
        }

        let new_octave = OrbOctave::build_octave(&octave_image,sigma, runtime_parameters);

        octaves.push(new_octave);
    }

    Pyramid {octaves}
}


pub fn generate_features_for_octave(octave: &OrbOctave, octave_idx: usize, runtime_parameters: &OrbRuntimeParameters) -> Vec<OrbFeature> {
    let orig_offset = runtime_parameters.fast_offsets;
    let scale = 2usize.pow(octave_idx as u32) as Float;
    let x_offset_scaled = (orig_offset.0 as Float / scale).trunc() as usize;
    let y_offset_scaled = (orig_offset.1 as Float / scale).trunc() as usize;

    OrbFeature::new(&octave.images, runtime_parameters.fast_circle_radius, runtime_parameters.fast_threshold_factor, runtime_parameters.fast_consecutive_pixels, runtime_parameters.fast_grid_size,(x_offset_scaled,y_offset_scaled), runtime_parameters.harris_k)
}

pub fn generate_feature_pyramid(pyramid: &Pyramid<OrbOctave>, runtime_parameters: &OrbRuntimeParameters) -> Pyramid<Vec<OrbFeature>> {
    Pyramid{octaves: pyramid.octaves.iter().enumerate().map(|(idx,x)| generate_features_for_octave(x,idx,runtime_parameters)).collect::<Vec<Vec<OrbFeature>>>()}
}


pub fn generate_feature_descriptor_pyramid(octave_pyramid: &Pyramid<OrbOctave>, feature_pyramid: &Pyramid<Vec<OrbFeature>>, sample_lookup_table: &Vec<Vec<(Point<Float>,Point<Float>)>>, runtime_parameters: &OrbRuntimeParameters) -> Pyramid<Vec<(OrbFeature,BriefDescriptor)>> {
    assert_eq!(octave_pyramid.octaves.len(),feature_pyramid.octaves.len());
    let octave_len = octave_pyramid.octaves.len();
    let mut feature_descriptor_pyramid = Pyramid::<Vec<(OrbFeature,BriefDescriptor)>>::empty(octave_len);

    //TODO: pyramids might not match for images takes at different scales, maybe n^2 would be better here, or maybe even no image pyramid for the target image!
    for i in 0..octave_len {
        let image = &octave_pyramid.octaves[i].images[0];
        let feature_octave = &feature_pyramid.octaves[i];
        let n = std::cmp::min(runtime_parameters.max_features_per_octave,feature_octave.len());
        let data_vector 
            = feature_octave.iter()
                            .enumerate()
                            .map(|x| (x.0,BriefDescriptor::new(image, x.1, runtime_parameters.brief_n, runtime_parameters.brief_s,&sample_lookup_table)))
                            .filter(|x| x.1.is_some())
                            .map(|(idx,option)| (feature_octave[idx],option.unwrap()))
                            .take(n)
                            .collect::<Vec<(OrbFeature,BriefDescriptor)>>();

        if data_vector.len() == 0 {
            println!("Warning: 0 features with descriptors for octave idx: {}",i);
        }

        feature_descriptor_pyramid.octaves.push(data_vector);
    }

    feature_descriptor_pyramid
}

pub fn generate_match_pyramid(feature_descriptor_pyramid_a: &Pyramid<Vec<(OrbFeature,BriefDescriptor)>>,feature_descriptor_pyramid_b: &Pyramid<Vec<(OrbFeature,BriefDescriptor)>>,  runtime_parameters: &OrbRuntimeParameters) -> Vec<((usize,OrbFeature),(usize,OrbFeature))> {

    let octave_levels_a = feature_descriptor_pyramid_a.octaves.iter().enumerate().map(|(i,x)| vec!(i;x.len())).flatten().collect::<Vec<usize>>();
    let octave_levels_b = feature_descriptor_pyramid_b.octaves.iter().enumerate().map(|(i,x)| vec!(i;x.len())).flatten().collect::<Vec<usize>>();

    let all_features_descriptors_a = feature_descriptor_pyramid_a.octaves.iter().map(|x| x.clone()).flatten().collect::<Vec<(OrbFeature,BriefDescriptor)>>();
    let all_features_descriptors_b = feature_descriptor_pyramid_b.octaves.iter().map(|x| x.clone()).flatten().collect::<Vec<(OrbFeature,BriefDescriptor)>>();


    let (all_features_a, all_descriptors_a): (Vec<OrbFeature>, Vec<BriefDescriptor>) = all_features_descriptors_a.into_iter().unzip();
    let (all_features_b, all_descriptors_b):  (Vec<OrbFeature>, Vec<BriefDescriptor>) = all_features_descriptors_b.into_iter().unzip();

    let matches_indices
        = BriefDescriptor::match_descriptors(&all_descriptors_a, &all_descriptors_b, runtime_parameters.brief_matching_min_threshold).iter().enumerate()
        .filter(|(_,option)| option.is_some())
        .map(|(idx,option)| (idx,option.unwrap()) ).collect::<Vec<(usize,usize)>>();

    let matches = matches_indices.into_iter().map(|(a_idx,b_idx)| ((octave_levels_a[a_idx],all_features_a[a_idx]),(octave_levels_b[b_idx],all_features_b[b_idx]))).collect::<Vec<((usize,OrbFeature),(usize,OrbFeature))>>();



    matches

}



