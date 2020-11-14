pub mod filter;
pub mod image;
pub mod pyramid;
pub mod features;
pub mod matching;
pub mod visualize;
pub mod vo;
pub mod numerics;
pub mod camera;

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


//TODO: maybe move this to image
pub fn reconstruct_original_coordiantes(x: usize, y: usize, octave_level: u32) -> (usize,usize) {
    let factor = 2usize.pow(octave_level);
    (x*factor,y*factor)
}



