use std::{sync::mpsc, thread};

use winit::{
    event::{Event},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let (tx, rx) = mpsc::channel::<Event<'static, ()>>();

    let _thread = thread::Builder::new().name(String::from("event loop")).spawn(move || {
        #[cfg(target_family = "unix")]
        let event_loop = <EventLoop<()> as winit::platform::unix::EventLoopExtUnix>::new_any_thread();
        #[cfg(target_family = "windows")]
        let event_loop = <EventLoop<()> as winit::platform::windows::EventLoopExtWindows>::new_any_thread();
        
        let _window = WindowBuilder::new().build(&event_loop).unwrap();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            if let Some(event) = event.to_static() {
                tx.send(event).unwrap();
            }
        });
    });

    for e in rx {
        println!("{:?}", e);
    }
}