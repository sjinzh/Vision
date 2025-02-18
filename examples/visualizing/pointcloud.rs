extern crate kiss3d;
extern crate nalgebra as na;
extern crate rand;

use std::{fs,result::Result};

use std::path::Path;
use kiss3d::event::{WindowEvent , Key};
use kiss3d::window::Window;
use kiss3d::text::Font;
use kiss3d::nalgebra::{Point2,Point3,Translation3};
use na::{Vector3, Isometry3};

use vision::{Float,load_runtime_conf};
use vision::sfm::{bundle_adjustment::state, landmark::{euclidean_landmark::EuclideanLandmark, inverse_depth_landmark::InverseLandmark}};
use rand::random;
use kiss3d::camera::ArcBall;

//TODO: render all states to screenshots

fn make_screenshot(file_path: &str, window: &Window) -> () {
    let image_buffer = window.snap_image();
    let img_path = Path::new(file_path);
    image_buffer.save(img_path).unwrap();
    println!("Screeshot saved to {}",file_path);  
}

fn clear_scene(window: &mut Window, scene_nodes: &mut Vec::<kiss3d::scene::SceneNode>) ->() {
    while scene_nodes.len() > 0 {
        let mut p = scene_nodes.pop();
        let node = p.as_mut().unwrap();
        window.remove_node(node);
    }
}

fn populate_scene(window: &mut Window, scene_nodes: &mut Vec::<kiss3d::scene::SceneNode>, cams: &Vec<Isometry3<Float>>, points: & Vec<Vector3<Float>>) -> () {
    for cam in cams {
        let cam_world = cam.inverse();
        let mut s = window.add_sphere(0.1);
        s.set_color(random(), random(), random());
        let factor = 1.0;
        s.append_translation(&Translation3::new(factor*(cam_world.translation.vector[0] as f32),factor*(cam_world.translation.vector[1] as f32),factor*(cam_world.translation.vector[2] as f32)));
        scene_nodes.push(s);
    }

    let point_factor = 1.0;
    for point in points {
        let mut s = window.add_sphere(0.02);
        s.set_color(random(), random(), random());
        s.append_translation(&Translation3::new(point_factor*(point[0] as f32), point_factor*(point[1] as f32),  point_factor*(point[2] as f32)));
        scene_nodes.push(s);
        
    }
}

fn main() -> Result<(),()> {
    let mut window = Window::new("BA: Pointcloud");
    let runtime_conf = load_runtime_conf();

    let final_state_as_string = fs::read_to_string(format!("{}/ba_synthetic.txt",runtime_conf.output_path)).expect("Unable to read file");
    let all_states_as_string_option = fs::read_to_string(format!("{}/ba_synthetic_debug.txt",runtime_conf.output_path));

/*     let final_state_as_string = fs::read_to_string(format!("{}/olsen.txt", runtime_conf.output_path)).expect("Unable to read file");
    let all_states_as_string_option = fs::read_to_string(format!("{}/olsen_debug.txt", runtime_conf.output_path)); */

    let loaded_state: (Vec<[Float;6]>,Vec<[Float;3]>) = serde_yaml::from_str(&final_state_as_string).unwrap();
    let ba_state = state::State::<Float,EuclideanLandmark<Float>,3>::from_serial(&loaded_state);

    let all_ba_states_option = match all_states_as_string_option {
        Ok(all_states_as_string) => {
            let loaded_all_states: Vec<(Vec<[Float;6]>,Vec<[Float;3]>)> = serde_yaml::from_str(&all_states_as_string).unwrap();
            Some(loaded_all_states.iter().map(|x|  state::State::<Float,EuclideanLandmark<Float>,3>::from_serial(x)).collect::<Vec< state::State::<Float,EuclideanLandmark<Float>,3>>>())
        },
        Err(_) => None
    };

    // let loaded_state: (Vec<[Float;6]>,Vec<[Float;6]>) = serde_yaml::from_str(&final_state_as_string).unwrap();
    // let ba_state = state::State::<InverseLandmark,6>::from_serial(&loaded_state);
    //let loaded_all_states: Vec<(Vec<[Float;6]>,Vec<[Float;6]>)> = serde_yaml::from_str(&all_states_as_string).unwrap();
    //let all_ba_states = loaded_all_states.iter().map(|x|  state::State::<InverseLandmark,6>::from_serial(x)).collect::<Vec< state::State::<InverseLandmark,6>>>();

    let (cams,points) = ba_state.as_matrix_point();


    let mut scene_nodes = Vec::<kiss3d::scene::SceneNode>::with_capacity(300);

    let mut recording_scene = false;
    let mut record_counter = 0;

    populate_scene(&mut window,&mut scene_nodes,&cams,&points);

    let num_points_text = format!(
        "Number of points: {}",
        cams.len() + points.len()
    );

    let at = Point3::new(0.0, 0.0, 1.0);
    let eye = Point3::origin();
    let mut arc_ball = ArcBall::new(eye, at);
    arc_ball.set_dist_step(5.0);

    while window.render_with_camera(&mut arc_ball) {
        window.draw_text(
            &num_points_text,
            &Point2::new(0.0, 20.0),
            60.0,
            &Font::default(),
            &Point3::new(1.0, 1.0, 1.0),
        );

        if recording_scene && all_ba_states_option.as_ref().is_some(){
            // the previously populated scene
            let file_name = format!("{}/screenshot_{}.png",runtime_conf.output_path,record_counter);
            make_screenshot(file_name.as_str(), &window);
            record_counter += 1;

            if record_counter == all_ba_states_option.as_ref().unwrap().len() {
                clear_scene(&mut window, &mut scene_nodes);
                populate_scene(&mut window,&mut scene_nodes,&cams,&points);
                println!("finished taking screenshots");  
                recording_scene = false;
                record_counter = 0;
            }
        }

        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::P, _, _) => make_screenshot(&format!("{}/screenshot.png",runtime_conf.output_path).as_str(), &window),
                WindowEvent::Key(Key::R, _, _) => {
                    clear_scene(&mut window, &mut scene_nodes);
                    populate_scene(&mut window,&mut scene_nodes,&cams,&points);
                },
                WindowEvent::Key(Key::T, _, _) => recording_scene = true,
                WindowEvent::Key(Key::S, _, _) => {
                    if all_ba_states_option.as_ref().is_some() {
                        let state = &all_ba_states_option.as_ref().unwrap()[0];
                        clear_scene(&mut window, &mut scene_nodes);
                        let (debug_cams,debug_points) = state.as_matrix_point();
                        populate_scene(&mut window,&mut scene_nodes,&debug_cams,&debug_points);
                    }

                },
                WindowEvent::Key(Key::Left, _, _) => {
                    arc_ball.set_up_axis_dir(-kiss3d::nalgebra::Vector3::x_axis());
                },
                WindowEvent::Key(Key::Right, _, _) => {
                    arc_ball.set_up_axis_dir(kiss3d::nalgebra::Vector3::x_axis());
                },
                WindowEvent::Key(Key::Up, _, _) => {
                    arc_ball.set_up_axis_dir(kiss3d::nalgebra::Vector3::y_axis());
                },
                WindowEvent::Key(Key::Down, _, _) => {
                    arc_ball.set_up_axis_dir(-kiss3d::nalgebra::Vector3::y_axis());
                },
                _ => ()
            }
        }



        if recording_scene &&  all_ba_states_option.is_some() {
            let state = &all_ba_states_option.as_ref().unwrap()[record_counter];
            clear_scene(&mut window, &mut scene_nodes);
            let (debug_cams,debug_points) = state.as_matrix_point();
            populate_scene(&mut window,&mut scene_nodes,&debug_cams,&debug_points);
        }

    }

    Ok(())
}


