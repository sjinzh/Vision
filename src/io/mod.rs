extern crate nalgebra as na;
extern crate image as image_rs;


use std::path::Path;
use std::fs::File;
use std::io::{BufReader,Read,BufRead, LineWriter, Write};
use std::option::Option;
use na::{RowDVector,DMatrix,Matrix4, Vector4, OMatrix, SimdValue, RealField, Dim, Scalar, DefaultAllocator, allocator::Allocator};
use crate::image::{Image,image_encoding::ImageEncoding};
use crate::image::features::geometry::point::Point;
use crate::sensors::camera::{Camera,perspective::Perspective};
use crate::{float,Float};


pub mod three_dv_loader;
pub mod eth_loader;
pub mod tum_loader;
pub mod d455_loader;
pub mod image_loading_parameters;
pub mod imu_loading_parameters;
pub mod octave_loader;
pub mod olsson_loader;


pub fn parse_to_float(string: &str, negate_value: bool) -> Float {
    let parts = string.trim().split("e").collect::<Vec<&str>>();
    let factor = match negate_value {
        true => -1.0,
        false => 1.0
    };

    match parts.len() {
        1 => factor * parts[0].parse::<Float>().unwrap(),
        2 => {
            let num = parts[0].parse::<Float>().unwrap();
            let exponent = parts[1].parse::<i32>().unwrap();
            factor * num*(10f64.powi(exponent) as Float)
        },
        _ => panic!("string malformed for parsing to float: {}", string)
    }
}

//TODO: make this a generic camera arg
pub fn load_depth_image_from_csv(file_path: &Path, negate_values: bool, invert_y: bool, width: usize, height: usize, scale: Float, normalize: bool, set_default_depth: bool, transform_camera_option: &Option<(&Matrix4<Float>,&Perspective<Float>)>) -> Image {
    let file = File::open(file_path).expect("load_depth_map failed");
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();

    let mut matrix = DMatrix::<Float>::zeros(height,width);
    let values = contents.trim().split(|c| c == ' ' || c == ',' || c=='\n').map(|x| parse_to_float(x.trim(),negate_values)).collect::<Vec<Float>>();
    let values_scaled = values.iter().map(|&x| x/scale).collect::<Vec<Float>>();
    assert_eq!(values_scaled.len(),height*width);

    for (idx,row) in values_scaled.chunks(width).enumerate() {
        let vector = RowDVector::<Float>::from_row_slice(row);
        let row_idx = match invert_y {
            true => height-1-idx,
            false => idx
        };
        matrix.set_row(row_idx,&vector);
    }

    //TODO: optimize this
    if transform_camera_option.is_some() {
        let mut new_matrix = DMatrix::<Float>::zeros(height,width);
        let (depth_camera_transfrom,camera) = transform_camera_option.unwrap();
        for r in 0..height {
            for c in 0..width {
                let depth = matrix[(r,c)];
                if depth != 0.0 {
                    let backprojected_point = camera.backproject(&Point::<Float>::new(c as Float + 0.5,r as Float + 0.5), depth);
                    let transformed_point = depth_camera_transfrom*Vector4::<Float>::new(backprojected_point[0],backprojected_point[1],backprojected_point[2],1.0);
                    let new_image_coords = camera.project(&transformed_point.fixed_rows::<3>(0));
                    if new_image_coords.is_some() {
                        let new_coords = new_image_coords.unwrap();
                        let x_usize = new_coords.x.trunc() as usize;
                        let y_usize = new_coords.y.trunc() as usize;
    
                        if x_usize < width && y_usize < height {
                            new_matrix[(y_usize,x_usize)] = depth;
                        } 
                    }
                }  
            }
        }

        matrix = new_matrix;

    }

    if set_default_depth {
        fill_matrix_with_default_depth(&mut matrix,negate_values);
    }
    Image::from_matrix(&matrix, ImageEncoding::F64, normalize)
}

pub fn load_image_as_gray(file_path: &Path, normalize: bool, invert_y: bool) -> Image {
    let gray_image = image_rs::open(&Path::new(&file_path)).expect("load_image failed").to_luma8();
    let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
    Image::from_gray_image(&gray_image, normalize, invert_y, Some(file_name))
}

pub fn load_depth_image(file_path: &Path, negate_values: bool, invert_y: bool, scale: Float, set_default_depth: bool) -> Image {
    let depth_image = image_rs::open(&Path::new(&file_path)).expect("load_image failed").to_luma16();
    let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
    let mut image = Image::from_depth_image(&depth_image,negate_values,invert_y, Some(file_name));
    image.buffer /= scale;
    if set_default_depth {
        fill_matrix_with_default_depth(&mut image.buffer,negate_values);
    }


    image
}

fn fill_matrix_with_default_depth(target: &mut DMatrix<Float>,negate_values: bool) {
    let extrema = match negate_values {
        true => target.min(),
        false => target.max()
    };

    for r in 0..target.nrows(){
        for c in 0..target.ncols(){
            if target[(r,c)] == 0.0 {
                target[(r,c)] = extrema;
            } 

        }
    }

}

// //TODO: can be moved into a more general place for list manipulation
pub fn closest_ts_index(ts: Float, list: &Vec<Float>) -> usize {
    let mut min_delta = float::MAX;
    let mut min_idx = list.len()-1;

    for (idx, target_ts) in list.iter().enumerate() {
        let delta = (ts-target_ts).abs();
        
        if delta < min_delta {
            min_delta = delta;
            min_idx = idx;
        } else {
            break;
        }    
    }

    min_idx
}

pub fn load_images(dir_path: &str, extension: &str) -> (Vec<Image>, Vec<String>) {
    assert_eq!(dir_path.chars().last().unwrap(),'/');
    let paths = std::fs::read_dir(format!("{}images",dir_path).as_str()).unwrap();

    let mut image_name_tuples = paths.map(|x| x.unwrap().path()).filter(|x| {
        match x.extension() {
            Some(v) => v.to_str().unwrap().ends_with(&extension.to_uppercase()) || v.to_str().unwrap().ends_with(&extension.to_lowercase()),
            _ => false
        }
    }).map(|x| (load_image_as_gray(x.as_path(),false,false),x.file_name().unwrap().to_str().unwrap().to_string())).collect::<Vec<(Image,String)>>();

    image_name_tuples.sort_unstable_by_key(|(_,name)| name.split(&['_', '.']).collect::<Vec<&str>>()[1].parse::<usize>().expect("Olsen: could not parse image name"));

    image_name_tuples.into_iter().unzip()
}

pub fn write_matrix_to_file<T, M, N>(matrix: &OMatrix<T,M,N>, folder_path: &str, file_name: &str) -> () 
    where 
        T: Scalar + RealField + Copy + SimdValue, 
        M: Dim, 
        N: Dim,
        DefaultAllocator: Allocator<T, M, N> {
    let file_path = format!("{}/{}",folder_path,file_name);
    let file = File::create(&file_path).expect(format!("write_matrix_to_file: {} could not be created!",&file_path).as_str());
    let mut file_line_writer = LineWriter::new(file);

    for row in matrix.row_iter() {
        let mut contents_as_str = String::new();
        for elem in row.iter() {
            let s = format!("{} ",elem);
            contents_as_str.push_str(&s);
        }
        let mut line = String::from(contents_as_str.trim());
        line.push('\n');
        file_line_writer.write_all(line.as_bytes()).expect("write_matrix_to_file: error writing matrix to file");
    }
}

pub fn load_matrix_from(folder_path: &str, file_name: &str) -> DMatrix<Float> {
    let file_path = format!("{}/{}",folder_path,file_name);
    let file = File::open(&file_path).expect(format!("load_matrix_from: {} could not be opened!",&file_path).as_str());
    let reader = BufReader::new(file);
    let mut matrix_vec = Vec::<Float>::new();
    let mut row_iter = 0;

    for line in reader.lines() {
        let line_contents = line.expect("load_matrix_from: line could no be read");
        let float_vec = line_contents.trim().split_whitespace().map(|x| parse_to_float(x,false));
        matrix_vec.extend(float_vec);
        row_iter +=1;
    }
    let ncols = matrix_vec.len() / row_iter;
    DMatrix::<Float>::from_vec(row_iter, ncols, matrix_vec)
}
