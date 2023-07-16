use three_d::*;
use winit::event_loop::EventLoop;

pub fn run_saver<S: 'static + ScreenSaverState>(
    title: &str,
    initializer: impl FnOnce(&Window<()>) -> S,
) {
    run_saver_full(title, |_| {}, |_, window| initializer(window))
}

pub fn run_saver_full<Y, S: 'static + ScreenSaverState>(
    title: &str,
    window_yoinker: impl FnOnce(&winit::window::Window) -> Y,
    initializer: impl FnOnce(Y, &Window<()>) -> S,
) {
    let (event_loop, winit_window) = create_winit_window(title);
    let yoinked = window_yoinker(&winit_window);
    let window = Window::from_winit_window(
        winit_window,
        event_loop,
        SurfaceSettings::default(),
    )
        .unwrap();

    let mut state = initializer(yoinked, &window);
    window.render_loop(move |frame_input| {
        if frame_input.events.iter().any(|e| matches!(e, Event::KeyRelease {
            kind: Key::Escape,
            handled: false,
            ..
        })) {
            return FrameOutput {
                exit: true,
                ..Default::default()
            };
        }

        state.tick(frame_input)
    });
}

pub fn create_winit_window(title: &str) -> (EventLoop<()>, winit::window::Window) {
    let event_loop = winit::event_loop::EventLoop::new();
    let winit_window = winit::window::WindowBuilder::new()
        .with_title(title)
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(
            event_loop.primary_monitor(),
        )))
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();
    winit_window.set_cursor_visible(false);
    (event_loop, winit_window)
}

pub trait ScreenSaverState {
    fn tick(&mut self, frame_input: FrameInput<()>) -> FrameOutput;
}
