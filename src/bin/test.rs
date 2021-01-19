fn main() {
    
}

struct Foo {
    vec: Vec<i32>,
}

impl Foo {
    fn bar(&self) {
        self.vec.push(16);
    }
}