use crate::features::geometry::{offset::Offset,point::Point};


pub mod circle;
pub mod point;
pub mod offset;

#[derive(Debug,Clone)]
pub struct Geometry {
    // pub x_center: usize,
    // pub y_center: usize,
    pub center: Point<usize>,
    pub offsets: Vec<Offset>
}

impl Geometry {
    pub fn get_points(&self) -> Vec<Point<usize>> {
        let mut points = Vec::<Point<usize>>::new();

        for offset in &self.offsets {
            let x = (self.center.x as isize + offset.x) as usize;
            let y = (self.center.y as isize + offset.y) as usize;
            points.push(Point{x,y});
        }

        points
    }

    pub fn points(x_center: usize, y_center: usize, offsets: &Vec<Offset>) -> Vec<Point<usize>> {
        let mut points = Vec::<Point<usize>>::new();

        for offset in offsets {
            let x = (x_center as isize + offset.x) as usize;
            let y = (y_center as isize + offset.y) as usize;
            points.push(Point{x,y});
        }

        points

    }
}