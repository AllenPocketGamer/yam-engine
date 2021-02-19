extern crate nalgebra as na;

use yamengine::*;

fn main() {
    let tranform_default = Transform2D::default();

    println!("angle: {}", tranform_default.to_homogeneous_3d());
}