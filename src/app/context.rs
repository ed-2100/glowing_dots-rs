use std::{io::Write as _, sync::Arc, time::SystemTime};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Window, WindowAttributes},
};

// The ordering of this struct is important to the program's shutdown process.
pub struct Context {
    time_last_print: SystemTime,
    time_last_draw: SystemTime,
    _time_start: SystemTime,

    window: Arc<Window>,
}

// Public methods
impl Context {
    /// Creates a new `Context`.
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Self::create_window(event_loop);
        let _size = Self::get_window_size(&window);

        let time_initial = SystemTime::now();

        Self {
            window,
            _time_start: time_initial,
            time_last_draw: time_initial,
            time_last_print: time_initial,
        }
    }

    /// Queues drawing of another frame.
    pub fn redraw(&mut self) {
        self.update_timing();

        self.window.request_redraw();
    }

    /// Resizes the window.
    pub fn resize(&mut self, _new_size: PhysicalSize<u32>) {
        self.window.request_redraw();
    }

    /// Toggles fullscreen mode.
    pub fn toggle_fullscreen(&self) {
        if self.window.fullscreen().is_some() {
            self.window.set_fullscreen(None);
        } else {
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
    }
}

// Private methods
impl Context {
    /// Creates a new window.
    fn create_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
        Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(LogicalSize {
                            width: 500,
                            height: 500,
                        })
                        .with_resizable(true)
                        .with_title("glowing_dots"),
                )
                .unwrap(),
        )
    }

    /// Gets the size of the window, ensuring that it is at least 1x1.
    fn get_window_size(window: &Window) -> PhysicalSize<u32> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        size
    }

    /// Updates the timing information for the context.
    fn update_timing(&mut self) {
        let time_current = SystemTime::now();
        if time_current.duration_since(self.time_last_print).unwrap()
            > std::time::Duration::from_millis(50)
        {
            print!(
                "\x1b[s{:7.1}\x1b[u",
                1.0 / time_current
                    .duration_since(self.time_last_draw)
                    .unwrap()
                    .as_secs_f32()
            );
            std::io::stdout().flush().unwrap();
            self.time_last_print = time_current;
        }
        self.time_last_draw = time_current;
    }
}
