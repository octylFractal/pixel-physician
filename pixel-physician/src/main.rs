use clap::{Parser, Subcommand};
use crate::savers::common::create_winit_window;
use crate::savers::mosaic_to_my_screen::MosaicToMyScreenStarter;

use crate::savers::spinny_cube::SpinnyCubeStarter;

mod savers;
pub(crate) mod desktop_capture;
pub(crate) mod util;
pub(crate) mod squaring;

#[derive(Parser, Debug)]
struct PixelPhysician {
    #[command(subcommand)]
    saver: ScreenSaverChoice,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "kebab-case")]
enum ScreenSaverChoice {
    /// A dummy choice that runs the screenshot code for testing.
    Screenshot,
    SpinnyCube {
        #[command(flatten)]
        inner: SpinnyCubeStarter,
    },
    MosaicToMyScreen {
        #[command(flatten)]
        inner: MosaicToMyScreenStarter,
    },
}

impl ScreenSaverChoice {
    fn run(self) {
        match self {
            ScreenSaverChoice::Screenshot => test_screen_capture(),
            ScreenSaverChoice::SpinnyCube { inner } => inner.run(),
            ScreenSaverChoice::MosaicToMyScreen { inner } => inner.run(),
        }
    }
}

fn test_screen_capture() {
    let (evt_loop, window) = create_winit_window("Pixel Physician Test");
    let screenshot_taker = crate::desktop_capture::create_screenshot_taker(&window);
    let initial_size = window.inner_size();
    window.set_visible(true);
    evt_loop.run(move |_, _, flow| {
        if window.inner_size().width > initial_size.width {
            let image = screenshot_taker.take_screenshot().expect("failed to take screenshot");
            println!("screenshot taken: {}x{}", image.width(), image.height());
            flow.set_exit();
        }
    });
}

pub fn main() {
    let args = PixelPhysician::parse();
    args.saver.run();
}
