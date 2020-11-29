extern crate nalgebra as na;
extern crate image as image_rs;

use na::{RowDVector,DMatrix};
use std::path::Path;
use std::fs::File;
use std::io::{BufReader,Read};

use crate::Float;
use crate::io::parse_to_float;
use crate::image::{Image,image_encoding::ImageEncoding};
use crate::camera::pinhole::Pinhole;

pub fn load_depth_image(file_path: &Path) -> Image {
    let file = File::open(file_path).expect("load_depth_map failed");
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();

    let rows = 480;
    let cols = 640;
    let mut matrix = DMatrix::<Float>::zeros(rows,cols);

    let values = contents.trim().split(" ").map(|x| parse_to_float(x)).collect::<Vec<Float>>();
    assert_eq!(values.len(),rows*cols);

    for (idx,row) in values.chunks(cols).enumerate() {
        let vector = RowDVector::<Float>::from_row_slice(row);
        matrix.set_row(idx,&vector);
    }

    Image::from_matrix(&matrix, ImageEncoding::F64, false)
}

pub fn load_image(file_path: &Path) -> Image {
    let gray_image = image_rs::open(&Path::new(&file_path)).expect("load_image failed").to_luma();
    Image::from_gray_image(&gray_image, false)
}

pub fn load_intrinsics(file_path: &Path) -> Pinhole {
    let file = File::open(file_path).expect("load_intrinsics failed");
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();
    let values = contents.trim().split(|c| !char::is_numeric(c)).map(|s| (s.len(),s.parse::<f64>())).filter(|(_,option)|  option.is_ok()).map(|(len,option)| (len,option.unwrap())).collect::<Vec<(usize,Float)>>();

    let fx = values[0].1 + values[1].1/(10f64.powi(values[1].0 as i32));
    let fy = values[6].1 +values[7].1/(10f64.powi(values[7].0 as i32));

    let cx = values[3].1;
    let cy = values[8].1;

    Pinhole::new(fx, fy, cx, cy)

}