extern crate nalgebra as na;

use core::f32;
use std::vec;

use na::{
    ArrayStorage, Dynamic, Isometry2, Isometry3, Matrix, Matrix3x4, Matrix4, Orthographic3, Perspective3, Point2, Point3, Similarity2,
    VecStorage, Vector2, Vector3, Vector4, U2, U3,
};

fn main() {}

fn about_matrix() {
    // Statically sized and statically allocated 2x3 matrix using 32-bit floats
    type Matrix2x3f = Matrix<f32, U2, U3, ArrayStorage<f32, U2, U3>>;

    // Hlaf-dynamically sized and dynamically allocated matrix with two rows using 64-bit floats
    type Matrix2xXf64 = Matrix<f64, U2, Dynamic, VecStorage<f64, U2, Dynamic>>;

    // Dynamically sized and dynamically allocated matrix with two rows and using 32-bit signed integers
    type DMatrixi32 = Matrix<i32, Dynamic, Dynamic, VecStorage<i32, Dynamic, Dynamic>>;

    // A vector with three components
    let v = Vector3::new(1, 2, 3);

    // A matrix with three lines and four columns
    // We chose values such taht, for example, 23 is at the row 2 and column 3
    let m = Matrix3x4::new(11, 12, 13, 14, 21, 22, 23, 24, 31, 32, 33, 34);
}

fn about_point() {
    // Build using components directly
    let p0 = Point3::new(2.0, 3.0, 4.0);

    // Build from a coordinates vector
    let coords = Vector3::new(2.0, 3.0, 4.0);
    let p1 = Point3::from(coords);

    // Build by translating the origin
    let translation = Vector3::new(2.0, 3.0, 4.0);
    let p2 = Point3::origin() + translation;

    // Build from homogeneous coordinates. The last component of the
    // vector will be remvoed and all other components divided by 10.0
    let homogeneous_coords = Vector4::new(20.0, 30.0, 40.0, 10.0);
    let p3 = Point3::from_homogeneous(homogeneous_coords).unwrap();

    assert_eq!(p0, p1);
    assert_eq!(p0, p2);
    assert_eq!(p0, p3);
}

fn about_transformation() {
    // Isometry -> Similarity conversion always succeeds
    let iso = Isometry2::new(Vector2::new(1.0f32, 2.0), na::zero());
    let _: Similarity2<f32> = na::convert(iso);

    // Similarity -> Isometry conversion fails if the scaling factor is not 1.0
    let sim_without_scaling = Similarity2::new(Vector2::new(1.0f32, 2.0), 3.14, 1.0);
    let sim_with_scaling = Similarity2::new(Vector2::new(1.0f32, 2.0), 3.14, 2.0);

    let iso_success: Option<Isometry2<f32>> = na::try_convert(sim_without_scaling);
    let iso_fail: Option<Isometry2<f32>> = na::try_convert(sim_with_scaling);

    assert!(iso_success.is_some());
    assert!(iso_fail.is_none());

    // Similarity -> Isometry conversion can be forced at your own risks
    let iso_forced: Isometry2<f32> = unsafe { na::convert_unchecked(sim_with_scaling) };
    assert_eq!(iso_success.unwrap(), iso_forced);
}

// The question: the relationship quaternion and rotation matrix

fn about_homogeneous() {
    // With dedicated transform types
    let iso = Isometry2::new(Vector2::new(1.0, 1.0), std::f32::consts::PI);
    let pt = Point2::new(1.0, 0.0);
    let vec = Vector2::<f32>::x();

    let transformed_pt = iso * pt;
    let transformed_vec = iso * vec;

    // Compute using homogeneous coordinates
    let hom_iso = iso.to_homogeneous();
    let hom_pt = pt.to_homogeneous();
    let hom_vec = vec.to_homogeneous();

    let hom_transformed_pt = hom_iso * hom_pt;
    let hom_transformed_vec = hom_iso * hom_vec;

    // Convert back to the cartesian coordinates
    let back_transformed_pt = Point2::from_homogeneous(hom_transformed_pt).unwrap();
    let back_transformed_vec = Vector2::from_homogeneous(hom_transformed_vec).unwrap();

    // TODO
    // assert_relative_eq!(transformed_pt, back_transformed_pt);
    // assert_relative_eq!(transformed_vec, back_transformed_vec);
}

fn about_orthographic_projection() {
    // Arguments order: left, right, bottom, top, znear, zfar
    let orth = Orthographic3::new(1.0, 2.0, -3.0, -2.5, 10.0, 900.0);
    let pt = Point3::new(1.0, -3.0, -10.0);
    let vec = Vector3::new(21.0, 0.0, 0.0);

    assert_eq!(orth.project_point(&pt), Point3::new(-1.0, -1.0, -1.0));
    assert_eq!(orth.project_vector(&vec), Vector3::new(42.0, 0.0, 0.0));
}

fn about_perspective_projection() {
    // Arguments order: aspect, fovy, znear, zfar
    let proj = Perspective3::new(16.0 / 9.0, 3.14 / 4.0, 1.0, 10000.0);
}

fn transformation_using_matrix4() {
    // Create a uniform scaling matrix with scaling factor 2
    let mut m = Matrix4::new_scaling(2.0);

    assert_eq!(m.transform_vector(&Vector3::x()), Vector3::x() * 2.0);
    assert_eq!(m.transform_vector(&Vector3::y()), Vector3::y() * 2.0);
    assert_eq!(m.transform_vector(&Vector3::z()), Vector3::z() * 2.0);

    // Append a non-uniform scaling in-place
    m.append_nonuniform_scaling_mut(&Vector3::new(1.0, 2.0, 3.0));

    assert_eq!(m.transform_vector(&Vector3::x()), Vector3::x() * 2.0);
    assert_eq!(m.transform_vector(&Vector3::y()), Vector3::y() * 4.0);
    assert_eq!(m.transform_vector(&Vector3::z()), Vector3::z() * 6.0);

    // Append a translation out-of-place
    let m2 = m.append_translation(&Vector3::new(42.0, 0.0, 0.0));

    assert_eq!(m2.transform_point(&Point3::new(1.0, 1.0, 1.0)), Point3::new(42.0 + 2.0, 4.0, 6.0));

    // Create rotation
    let rot = Matrix4::from_scaled_axis(&Vector3::x() * 3.14);
    let rot_then_m = m * rot;
    let m_then_rot = rot * m;

    let pt = Point3::new(1.0, 2.0, 3.0);

    // TODO
    // assert_relative_eq!(m.transform_point(&rot.transform_point(&pt)), rot_then_m.transform_point(&pt));
    // assert_relative_eq!(rot.transform_point(&m.transform_point(&pt)), m_then_rot.transform_point(&pt));
}
