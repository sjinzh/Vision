
use crate::features::geometry::{offset::Offset,point::Point};


#[derive(Debug,Clone)]
pub struct Circle {
    pub x_center: usize,
    pub y_center: usize,
    pub offsets: Vec<Offset>
}

impl Circle {
    pub fn points(&self) -> Vec<Point> {
        let mut points = Vec::<Point>::new();

        for offset in &self.offsets {
            let x = (self.x_center as isize + offset.x) as usize;
            let y = (self.y_center as isize + offset.y) as usize;
            points.push(Point{x,y});
        }

        points
    }
}

// https://www.geeksforgeeks.org/bresenhams-circle-drawing-algorithm/?ref=rp
pub fn circle_bresenham(x_center: usize, y_center: usize, radius: usize) -> Circle {

    let mut x: isize = 0;
    let mut y: isize = radius as isize;
    let mut d = 3 -2*radius as isize;

    let mut circle = Circle{x_center,y_center,offsets: Vec::<Offset>::new()};
    
    circle.offsets.extend(bresenham_octant(x,y));
    while y >= x {
        x+=1; 
        if d > 0 {
            y-=1;
            d = d + 4*(x as isize-y as isize) +10;
        } else {
            d = d + 4*(x as isize)+6;
        }
        circle.offsets.extend(bresenham_octant(x,y));

    }

    circle
}

fn bresenham_octant( x: isize, y: isize) -> Vec<Offset> {

    let start = -1;
    let x_end : isize = if x == 0 {0} else {2};
    let y_end: isize = 2;
    let mut points = Vec::<Offset>::new();

    for x_sign in (start..x_end).step_by(2) {
        for y_sign in (start..y_end).step_by(2) {
            let x_signed = x_sign*x;
            let y_signed = y_sign*y;

            points.push(Offset{x: x_signed,y: y_signed });
            if x != y {
                points.push(Offset{x: y_signed,y: x_signed });
            }
        }
    }
    


    points
}