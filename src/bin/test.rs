use std::{borrow::Borrow, path::Path};


fn main() {
    let foo = WithoutCopy(0);
    let mut bar = foo;
    bar.0 = 12;
}

#[derive(Debug)]
struct WithoutCopy(i32);

#[derive(Debug)]
struct Container {
    foo: WithoutCopy,
    bar: WithoutCopy
}