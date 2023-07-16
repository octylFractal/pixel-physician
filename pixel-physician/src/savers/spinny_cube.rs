use clap::Args;
use three_d::*;

use crate::savers::common::{run_saver, ScreenSaverState};
use crate::util::{compute_sliding, lights_vec};

#[derive(Debug, Args)]
pub struct SpinnyCubeStarter;

impl SpinnyCubeStarter {
    pub fn run(self) {
        run_saver("Spinny Cube", |window| {
            let context = window.gl();

            let camera = Camera::new_perspective(
                window.viewport(),
                vec3(0.0, 0.0, 5.5),
                vec3(0.0, 0.0, -0.5),
                vec3(0.0, 1.0, 0.0),
                degrees(45.0),
                0.1,
                1000.0,
            );

            let mut cube = Gm::new(
                Mesh::new(&context, &CpuMesh::cube()),
                PhysicalMaterial::new_transparent(
                    &context,
                    &CpuMaterial {
                        albedo: Color {
                            r: 0,
                            g: 0,
                            b: 255,
                            a: 100,
                        },
                        ..Default::default()
                    },
                ),
            );
            cube.set_transformation(Mat4::from_scale(3.5));
            cube.set_animation(Box::new(|time| Mat4::from_angle_y(Rad(time * 0.001))));

            let light0 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, -0.5, -0.5));
            let light1 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, 0.5, 0.5));

            SpinnyCubeState {
                camera,
                cube,
                lights: [light0, light1],
            }
        });
    }
}

struct SpinnyCubeState {
    camera: Camera,
    cube: Gm<Mesh, PhysicalMaterial>,
    lights: [DirectionalLight; 2],
}

impl ScreenSaverState for SpinnyCubeState {
    fn tick(&mut self, frame_input: FrameInput<()>) -> FrameOutput {
        let cube = &mut self.cube;
        cube.animate(frame_input.accumulated_time as f32);
        let pre_val = (frame_input.accumulated_time * 0.01) as u32 % 512;
        cube.material.albedo.r = compute_sliding(pre_val);
        // offset the two colors to make it more interesting
        cube.material.albedo.g = compute_sliding((pre_val + 128) % 512);
        cube.material.albedo.b = compute_sliding((pre_val + 64) % 512);
        self.camera.set_viewport(frame_input.viewport);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &self.camera,
                self.cube.into_iter(),
                &lights_vec(&self.lights),
            );

        FrameOutput::default()
    }
}
