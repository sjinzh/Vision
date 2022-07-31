extern crate nalgebra as na;
extern crate num_traits;
extern crate simba;

use std::collections::HashMap;
use std::marker::PhantomData;
use crate::sensors::camera::Camera;
use crate::image::features::{Feature, Match};


use num_traits::{float,NumAssign};
use na::{SimdRealField, ComplexField,base::Scalar, RealField};

pub mod bundle_adjustment;
pub mod landmark; 


macro_rules! define_sfm_float {
    ($f:tt) => {
        pub use std::$f as sfm_float;
        pub type SfmFloat = $f;
    }
}
define_sfm_float!(f32);

/**
 * We assume that the indices between paths and matches are consistent
 */
pub struct SFMConfig<F: float::Float + Scalar + NumAssign + SimdRealField + ComplexField + RealField, C: Camera<F>, Feat: Feature> {
    root: usize,
    paths: Vec<Vec<usize>>,
    camera_map: HashMap<usize, C>,
    matches: Vec<Vec<Vec<Match<Feat>>>>,
    phantom: PhantomData<fn(F) -> F>
}

impl<F: float::Float + Scalar + NumAssign + SimdRealField + ComplexField + RealField, C: Camera<F>, Feat: Feature> SFMConfig<F,C,Feat> {

    pub fn new(root: usize, paths: Vec<Vec<usize>>, camera_map: HashMap<usize, C>, matches: Vec<Vec<Vec<Match<Feat>>>>) -> SFMConfig<F,C,Feat> {
        SFMConfig{root, paths, camera_map, matches, phantom: PhantomData }
    }

    pub fn root(&self) -> usize { self.root }
    pub fn paths(&self) -> &Vec<Vec<usize>> { &self.paths }
    pub fn camera_map(&self) -> &HashMap<usize, C> { &self.camera_map }
    pub fn matches(&self) -> &Vec<Vec<Vec<Match<Feat>>>> { &self.matches } // TODO: These are not the filtered matches which are usually what are used. Unify this

    pub fn compute_path_id_pairs(&self) -> Vec<Vec<(usize, usize)>> {
        let mut path_id_paris = Vec::<Vec::<(usize,usize)>>::with_capacity(self.paths.len());
        for sub_path in &self.paths {
            path_id_paris.push(
                sub_path.iter().enumerate().map(|(i,&id)| 
                    match i {
                        0 => (self.root,id),
                        idx => (sub_path[idx-1],id)
                    }
                ).collect()
            )
        }

        path_id_paris
    }

    pub fn compute_unqiue_ids_cameras_root_first(&self) -> (Vec<usize>, Vec<&C>) {
        let number_of_keys = self.camera_map.keys().len();
        let mut keys_sorted = Vec::<usize>::with_capacity(number_of_keys);
        // root has to first by design
        keys_sorted.push(self.root());
        keys_sorted.extend(self.paths.clone().into_iter().flatten().collect::<Vec<usize>>());
        keys_sorted.dedup();
        let cameras_sorted = keys_sorted.iter().map(|id| self.camera_map.get(id).expect("compute_unqiue_ids_cameras_sorted: trying to get invalid camera")).collect::<Vec<&C>>();
        (keys_sorted,cameras_sorted)
    }

}