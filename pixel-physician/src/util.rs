use image::RgbImage;
use three_d::{CpuTexture, Light};
use three_d_asset::TextureData;

pub fn compute_sliding(pre_val: u32) -> u8 {
    if pre_val < 256 {
        pre_val as u8
    } else {
        (511 - pre_val) as u8
    }
}

pub fn lights_vec<L: Light>(lights: &[L]) -> Vec<&dyn Light> {
    lights.iter().map(|l| l as &dyn Light).collect()
}

pub trait IntoTexture {
    fn into_texture(self) -> CpuTexture;
}

impl IntoTexture for RgbImage {
    fn into_texture(self) -> CpuTexture {
        let width = self.width();
        let height = self.height();
        CpuTexture {
            name: "derived from image".to_string(),
            data: TextureData::RgbU8(self.pixels().map(|p| p.0).collect()),
            width,
            height,
            ..Default::default()
        }
    }
}
