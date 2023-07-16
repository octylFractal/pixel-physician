use clap::Args;
use humantime::Duration;
use image::{DynamicImage, GenericImageView, RgbImage};
use rayon::iter::{IntoParallelIterator, ParallelIterator, IndexedParallelIterator};
use three_d::*;

use crate::desktop_capture::{create_screenshot_taker, ScreenshotTaker};
use crate::savers::common::{run_saver_full, ScreenSaverState};
use crate::squaring::{SectionedImage, Size};
use crate::util::IntoTexture;

#[derive(Debug, Args)]
pub struct MosaicToMyScreenStarter {
    /// Length of time to display a level.
    #[arg(short, long, default_value = "1s")]
    pub display_time: Duration,
}

impl MosaicToMyScreenStarter {
    pub fn run(self) {
        run_saver_full(
            "Mosaic to My Screen",
            create_screenshot_taker,
            move |screenshot_taker: ScreenshotTaker, window| {
                let context = window.gl();
                let camera = Camera::new_orthographic(
                    window.viewport(),
                    vec3(0.0, 0.0, 1.0),
                    vec3(0.0, 0.0, -0.5),
                    vec3(0.0, 1.0, 0.0),
                    // initial height is bogus, we'll be setting it later
                    1.0,
                    0.1,
                    1000.0,
                );

                let screen = Gm::new(
                    Mesh::new(&context, &CpuMesh::square()),
                    ColorMaterial::new(
                        &context,
                        &Default::default(),
                    ),
                );

                let light = DirectionalLight::new(
                    &context, 1.0, Color::WHITE, &vec3(0.0, 0.0, -1.0),
                );

                MosaicToMyScreenState {
                    args: self,
                    screenshot_taker,
                    mosaic_state: MosaicState::TransparentDraw {
                        remaining_frames: 30,
                    },
                    camera,
                    screen,
                    light,
                }
            },
        );
    }
}

fn reset_camera_projection(camera: &mut Camera, width: f32, height: f32) {
    let viewport = camera.viewport();
    let width_ratio = width / viewport.width as f32;
    let height_ratio = height / viewport.height as f32;
    let viewport_height = width_ratio.max(height_ratio) * viewport.height as f32;
    camera.set_orthographic_projection(
        viewport_height,
        0.1,
        1000.0,
    );
}

struct MosaicToMyScreenState {
    args: MosaicToMyScreenStarter,
    screenshot_taker: ScreenshotTaker,
    camera: Camera,
    mosaic_state: MosaicState,
    screen: Gm<Mesh, ColorMaterial>,
    light: DirectionalLight,
}

impl MosaicToMyScreenState {
    fn pulse_mosaic(&mut self, frame_input: &FrameInput<()>) -> Option<FrameOutput> {
        let mut ret = None;
        self.mosaic_state = match std::mem::replace(&mut self.mosaic_state, MosaicState::TransparentDraw { remaining_frames: 0 }) {
            MosaicState::TransparentDraw { mut remaining_frames } => {
                remaining_frames = remaining_frames.saturating_sub(1);
                ret = Some(FrameOutput::default());
                if remaining_frames == 0 {
                    MosaicState::CaptureAndRender {
                        remaining_time: self.args.display_time.as_millis() as f64,
                    }
                } else {
                    MosaicState::TransparentDraw { remaining_frames }
                }
            }
            MosaicState::CaptureAndRender { mut remaining_time } => {
                remaining_time -= frame_input.elapsed_time;

                let capture = self.screenshot_taker.take_screenshot().expect("failed to load desktop screen");

                // scale to the image size
                let (width, height) = (capture.width() as f32, capture.height() as f32);
                self.screen.set_transformation(
                    Mat4::from_nonuniform_scale(width / 2.0, height / 2.0, 1.0)
                );

                let sizes: Vec<Size> = (0..u32::MAX)
                    .map_while(|level| get_tile_size(&capture, level))
                    .collect();
                let levels: Vec<RgbImage> = (0..sizes.len())
                    .into_par_iter()
                    .rev()
                    .map(|idx| mosaify(&capture, sizes[idx]))
                    .collect();

                assert!(!levels.is_empty(), "no levels found");

                ret = Some(FrameOutput {
                    swap_buffers: false,
                    ..Default::default()
                });
                if remaining_time <= 0.0 {
                    MosaicState::DisplayingMosaic {
                        wrote_level: false,
                        remaining_time: self.args.display_time.as_millis() as f64,
                        levels,
                        level: 0,
                    }
                } else {
                    MosaicState::WaitForTick {
                        remaining_time,
                        levels,
                    }
                }
            }
            MosaicState::WaitForTick { mut remaining_time, levels } => {
                remaining_time -= frame_input.elapsed_time;
                ret = Some(FrameOutput {
                    swap_buffers: false,
                    ..Default::default()
                });
                if remaining_time <= 0.0 {
                    MosaicState::DisplayingMosaic {
                        wrote_level: false,
                        remaining_time: self.args.display_time.as_millis() as f64,
                        levels,
                        level: 0,
                    }
                } else {
                    MosaicState::WaitForTick {
                        remaining_time,
                        levels,
                    }
                }
            }
            MosaicState::DisplayingMosaic { mut wrote_level, mut remaining_time, levels, level } => {
                remaining_time -= frame_input.elapsed_time;

                let image = &levels[level];
                if !wrote_level {
                    self.screen.material = ColorMaterial::new(
                        &frame_input.context,
                        &CpuMaterial {
                            albedo_texture: Some(image.clone().into_texture()),
                            ..Default::default()
                        },
                    );
                    wrote_level = true;
                }

                // update camera just in case
                reset_camera_projection(&mut self.camera, image.width() as f32, image.height() as f32);

                if remaining_time <= 0.0 {
                    if level + 1 < levels.len() {
                        MosaicState::DisplayingMosaic {
                            wrote_level: false,
                            remaining_time: self.args.display_time.as_millis() as f64,
                            levels,
                            level: level + 1,
                        }
                    } else {
                        MosaicState::TransparentDraw {
                            remaining_frames: 30,
                        }
                    }
                } else {
                    MosaicState::DisplayingMosaic {
                        wrote_level,
                        remaining_time,
                        levels,
                        level,
                    }
                }
            }
        };
        ret
    }
}

impl ScreenSaverState for MosaicToMyScreenState {
    fn tick(&mut self, frame_input: FrameInput<()>) -> FrameOutput {
        self.camera.set_viewport(frame_input.viewport);

        frame_input.screen().clear(
            ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0)
        );

        if let Some(output) = self.pulse_mosaic(&frame_input) {
            return output;
        }

        frame_input
            .screen()
            .render(
                &self.camera,
                self.screen.into_iter(),
                &[&self.light],
            );

        FrameOutput::default()
    }
}

#[derive(Debug)]
enum MosaicState {
    /// Draw the screen transparent for a few frames to allow the window to clear out.
    TransparentDraw {
        remaining_frames: usize,
    },
    /// Screen is transparent and we are capturing the desktop, then computing
    /// the mosaic levels.
    CaptureAndRender {
        remaining_time: f64,
    },
    /// The mosaic levels were computed before `remaining_time` elapsed.
    /// We need to wait for the remaining time.
    WaitForTick {
        remaining_time: f64,
        levels: Vec<RgbImage>,
    },
    /// The mosaic levels were computed and we are displaying them.
    /// Currently displaying level [level].
    /// When `remaining_time` elapses, we will display level [level + 1].
    /// When we reach the last level, we will wait for `remaining_time` and
    /// then go back to `TransparentDrawSubmitted`.
    DisplayingMosaic {
        wrote_level: bool,
        remaining_time: f64,
        levels: Vec<RgbImage>,
        level: usize,
    },
}

fn mosaify(source: &RgbImage, size: Size) -> RgbImage {
    // Slice out tiles
    let sectioned = SectionedImage {
        texture: source,
        section_size: size,
    };
    let tiles: Vec<DynamicImage> = sectioned.section_coords()
        .map(|c| sectioned.get_section(c).into())
        .collect();

    let (orig_width, orig_height) = (source.width(), source.height());

    // scale down the image so that width * tile_width >= orig_width, etc.
    let smaller_image = image::imageops::resize(
        source,
        (orig_width + size.width - 1) / size.width,
        (orig_height + size.height - 1) / size.height,
        image::imageops::FilterType::Nearest,
    );

    let mosaic = pixel_physician_tilr::Mosaic::new(
        smaller_image,
        tiles,
        size.width,
        size.height,
    );

    let mut mosaic = mosaic.into_image();

    if mosaic.width() != orig_width || mosaic.height() != orig_height {
        mosaic = mosaic.view(0, 0, orig_width, orig_height).to_image();
    }

    mosaic
}

fn get_tile_size(source: &RgbImage, level: u32) -> Option<Size> {
    let tile_count = 2u32.pow(level + 1);
    let tile_width = source.width() / tile_count;
    let tile_height = source.height() / tile_count;
    Some(Size {
        width: tile_width,
        height: tile_height,
    }).filter(|s| s.area() > 4)
}
