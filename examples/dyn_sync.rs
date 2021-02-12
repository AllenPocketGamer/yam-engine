use legion::*;
use systems::Runnable;

fn main() {
    let foo: Box<dyn Runnable + Send + Sync> = Box::new(temp_system());
    let bar: Box<dyn Runnable> = foo;
    // let foo: Box<dyn Runnable + Send + Sync> = bar;
}

#[system]
fn temp() {

}