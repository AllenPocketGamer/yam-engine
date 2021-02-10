use na::{Quaternion, Rotation3, Translation3, Vector3};

extern crate nalgebra as na;

fn main() {
    let axis = na::Vector3::<f32>::y_axis();
    let angle = 45.0 / 180.0 * std::f32::consts::PI;

    let rotation = na::Rotation3::from_axis_angle(&axis, angle);
    let quaternion = na::UnitQuaternion::<f32>::identity();

    println!("{}, size: {}", &rotation.to_homogeneous(), std::mem::size_of_val(&rotation));
    println!("{}, size: {}", &quaternion.to_homogeneous(), std::mem::size_of_val(&quaternion));

    let translation = na::Translation3::from(Vector3::new(1f32, 1.0, 1.0));

    println!("{}, size: {}", translation.to_homogeneous(), std::mem::size_of_val(&translation));
}