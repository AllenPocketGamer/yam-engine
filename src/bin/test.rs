use std::{borrow::Borrow, fmt::Display, hash::Hash, path::Path};


fn main() {
    let foo: &'static WithoutCopy = &WithoutCopy(12);

    let bar = WithoutCopy(11);
}

fn take<T>(t: &'static T) -> &'static T {
    t
}

struct WithoutCopy(i32);