use std::os::raw::c_ulong;

use image::{Rgb, RgbImage};
use winit::platform::x11::WindowExtX11;
use x11rb::connection::Connection;
use x11rb::image::{Image, PixelLayout};
use x11rb::protocol::xproto::ConnectionExt;
use x11rb_protocol::protocol::xproto::Window;

use crate::desktop_capture::{ScreenshotError, ScreenshotTakerImpl};

pub(super) fn create_screenshot_taker(window: &winit::window::Window) -> impl ScreenshotTakerImpl {
    if let Some(window_id) = window.xlib_window() {
        X11ScreenshotTaker { window_id }
    } else {
        panic!("X11 window not available");
    }
}

#[derive(Debug)]
struct X11ScreenshotTaker {
    window_id: c_ulong,
}

impl ScreenshotTakerImpl for X11ScreenshotTaker {
    fn take_screenshot(&self) -> Result<RgbImage, ScreenshotError> {
        let (connection, _screen_num) = x11rb::connect(None)?;
        let window: Window = self.window_id.try_into().expect("Window ID is too large");
        let geometry = connection.get_geometry(window)?.reply()?;
        let root_coords = connection.translate_coordinates(
            window,
            geometry.root,
            geometry.x,
            geometry.y,
        )?.reply()?;
        let (x11_image, visual_id) = Image::get(
            &connection,
            geometry.root,
            root_coords.dst_x,
            root_coords.dst_y,
            geometry.width,
            geometry.height,
        )?;
        let screen = connection.setup().roots.iter()
            .find(|screen| screen.root == geometry.root).expect("Couldn't find screen");
        let format = screen.allowed_depths.iter()
            .find_map(|d|
                d.visuals.iter().find(|v| v.visual_id == visual_id).copied()
            )
            .expect("visualid was not found in the screen's allowed depths");
        let layout = PixelLayout::from_visual_type(format)
            .expect("only can use TrueColor or DirectColor visuals");
        let mut image = RgbImage::new(
            x11_image.width().into(), x11_image.height().into(),
        );
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let x11_pixel = layout.decode(x11_image.get_pixel(x as u16, y as u16));
            *pixel = Rgb([
                (x11_pixel.0 >> 8) as u8,
                (x11_pixel.1 >> 8) as u8,
                (x11_pixel.2 >> 8) as u8,
            ]);
        }
        Ok(image)
    }
}
