#[cfg(unix)]
mod unix;

use std::fmt::Debug;
use thiserror::Error;

trait ScreenshotTakerImpl: Debug {
    fn take_screenshot(&self) -> Result<image::RgbImage, ScreenshotError>;
}

#[derive(Debug)]
pub struct ScreenshotTaker {
    inner: Box<dyn ScreenshotTakerImpl>,
}

impl ScreenshotTaker {
    pub fn take_screenshot(&self) -> Result<image::RgbImage, ScreenshotError> {
        self.inner.take_screenshot()
    }
}

pub fn create_screenshot_taker(window: &winit::window::Window) -> ScreenshotTaker {
    #[cfg(unix)]
    use unix::create_screenshot_taker as platform_create_screenshot_taker;

    ScreenshotTaker { inner: Box::new(platform_create_screenshot_taker(window)) }
}

/// Errors from a screencap.
#[derive(Error, Debug)]
pub enum ScreenshotError {
    #[error("X11 connect error: {0}")]
    X11Connect(#[from] x11rb::errors::ConnectError),
    #[error("X11 connection error: {0}")]
    X11Connection(#[from] x11rb::errors::ConnectionError),
    #[error("X11 request error: {0}")]
    X11Request(#[from] x11rb::errors::ReplyError),
    #[error("Image decoding error: {0}")]
    ImageError(#[from] image::ImageError),
}
