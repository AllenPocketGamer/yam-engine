use std::{cell::{Ref, RefCell}, collections::hash_map::DefaultHasher};

fn main() {
    let vec: Vec<NoDefault> = Default::default();

}

struct NoDefault;