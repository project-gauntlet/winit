#![allow(clippy::single_match)]

use simple_logger::SimpleLogger;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();

    // Allow us to drop the window.
    let mut window = Some(window);

    let mut timer = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Check if we're polling (waiting mode) and that enough time has elapsed.
        if *control_flow == winit::event_loop::ControlFlow::Poll
            && timer.elapsed() > std::time::Duration::from_millis(300)
        {
            // Now, we can safely request to exit the loop and drop the resources.
            control_flow.set_exit();
        }

        match event {
            // This event is emitted once, at the start of the application.
            Event::Resumed => {
                // Set `control_flow` to `wait` only once.
                control_flow.set_wait();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                // We'll be polling our own timer from now on.
                control_flow.set_poll();
                timer = std::time::Instant::now();

                // Drop the window (starts the animation).
                window.take();
            }
            Event::MainEventsCleared => {
                window.as_ref().map(|w| w.request_redraw());
            }
            _ => (),
        }
    });
}
