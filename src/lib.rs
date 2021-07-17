pub mod filter;
pub mod image;
pub mod pyramid;
pub mod features;
pub mod matching;
pub mod visualize;
pub mod odometry;
pub mod numerics;
pub mod io;
pub mod sensors;
pub mod bundle_adjustment;

macro_rules! define_float {
    ($f:tt) => {
        pub use std::$f as float;
        pub type Float = $f;
    }
}

define_float!(f64);

#[repr(u8)]
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum GradientDirection {
    HORIZINTAL,
    VERTICAL,
    SIGMA
}


//TODO: maybe move this to pyramid
pub fn reconstruct_original_coordiantes(x: usize, y: usize, base: usize, octave_index: u32) -> (usize,usize) {
    let factor = base.pow(octave_index);
    (x*factor,y*factor)
}

pub fn reconstruct_original_coordiantes_for_float(x: Float, y: Float, base:Float,  octave_index: i32) -> (Float,Float) {
    let factor = base.powi(octave_index);
    (x*factor,y*factor)
}




