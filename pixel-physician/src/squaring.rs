use cgmath::{ElementWise, Vector2};
use image::{GenericImage, GenericImageView, RgbImage};

#[derive(Debug)]
pub struct SectionedImage<'a> {
    pub texture: &'a RgbImage,
    pub section_size: Size,
}

impl<'a> SectionedImage<'a> {
    pub fn section_count(&self) -> Vector2<u32> {
        let width = (self.texture.width() + self.section_size.width - 1) / self.section_size.width;
        let height = (self.texture.height() + self.section_size.height - 1) / self.section_size.height;
        Vector2::new(width, height)
    }

    pub fn section_coords(&self) -> impl Iterator<Item=Vector2<u32>> {
        let sec_count = self.section_count();
        (0..sec_count.y).flat_map(move |y| {
            (0..sec_count.x).map(move |x| {
                Vector2::new(x, y)
            })
        })
    }

    pub fn get_section(&self, coord: Vector2<u32>) -> RgbImage {
        let base_coords = coord.mul_element_wise(self.section_size.into_vector());
        // copy into a new image with the correct size
        let mut new_image = RgbImage::new(self.section_size.width, self.section_size.height);
        new_image.copy_from(&*self.texture.view(
            base_coords.x,
            base_coords.y,
            self.section_size.width.min(self.texture.width() - base_coords.x),
            self.section_size.height.min(self.texture.height() - base_coords.y),
        ), 0, 0).expect("bounds calculation is wrong");
        new_image
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    pub fn into_vector(self) -> Vector2<u32> {
        Vector2::new(self.width, self.height)
    }
}
