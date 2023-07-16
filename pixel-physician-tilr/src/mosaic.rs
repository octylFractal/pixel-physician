// tilr - A program to build an image from a set of image 'tiles'.
// Copyright (C) 2023  Charles German <5donuts@pm.me>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::tiles::*;
use image::{DynamicImage, GenericImage, RgbImage};

/// Generates an image 'mosaic' using a set of image Tiles.
///
/// An image 'mosaic' is an image made up of a number of smaller
/// images in place of pixels. Using the average color of an image
/// Tile, a suitable large image mosaic viewed from far enough
/// away can appear to be a normal image.
#[allow(missing_debug_implementations)]
pub struct Mosaic {
    /// The original image used to create the mosaic.
    img: RgbImage,
    /// The set of [`Tile`]s to use to build the mosaic.
    ///
    /// Pixels in the original image are mapped to these tiles based
    /// on the Euclidean distance between the RGB pixel values and the
    /// average RGB values in the [`Tile`].
    tiles: TileSet,
    /// An inner member used to build the resulting image mosaic.
    inner: Inner,
}

impl Mosaic {
    /// Initialize a new image mosaic.
    ///
    /// # Arguments
    /// * `img` - The original image used to create the mosaic.
    /// * `tiles` - The set of Tiles to use to build the mosaic.
    /// * `img_scaling` - The scaling factor to apply to the original
    ///                   image for the mosaic. A scaling factor of `1`
    ///                   means no scaling. The scaling performed does
    ///                   _not_ preserve aspect ratio.
    /// * `tile_size` - The desired side length for the Tiles to use
    ///                 to generate this mosaic. If the Tiles are not
    ///                 already squares with this side length, they will
    ///                 be resized (without preserving aspect ratio) to
    ///                 be squares with the given side length.
    ///
    /// # Returns
    /// An empty mosaic. To build the mosaic, call [Mosaic::into_image].
    /// Note that generating the resulting mosaic is an expensive operation and
    /// could take many seconds (or minutes for especially large mosaics).
    pub fn new(
        img: RgbImage,
        tiles: Vec<DynamicImage>,
        tile_width: u32,
        tile_height: u32,
    ) -> Self {
        // Build the tileset
        let tiles = TileSet::from(tiles);

        // Initialize the inner image (the output mosaic image)
        let (img_x, img_y) = img.dimensions();
        let (mos_x, mos_y) = (img_x * tile_width, img_y * tile_height);
        let inner = Inner(RgbImage::new(mos_x, mos_y));

        Self { img, tiles, inner }
    }

    /// Generate the image mosaic and convert it to an [`RgbImage`].
    ///
    /// Depending on the size of the mosaic to build, this function may
    /// take some time to run.
    pub fn into_image(self) -> RgbImage {
        let map = self.tiles.map_to(&self.img);
        let (img_x, img_y) = self.img.dimensions();
        let tile_width = self.tiles.tile_x_len();
        let tile_height = self.tiles.tile_y_len();
        let mut mosaic = self.inner;

        // Build the mosaic
        let mut mos_x = 0;
        for x in 0..img_x {
            let mut mos_y = 0;
            for y in 0..img_y {
                // Add the tile to the mosaic
                let tile_for_px = map.get(&self.img.get_pixel(x, y)).expect("No tile for px");
                mosaic.add_tile(tile_for_px, (mos_x, mos_y));

                // Move to the next row in the mosaic
                mos_y += tile_height;
            }

            // Move to the next col in the mosaic
            mos_x += tile_width;
        }

        mosaic.0
    }
}

/// A wrapper around a [`DynamicImage`] used to build the resulting
/// image mosaic.
struct Inner(RgbImage);

impl Inner {
    /// Add a [`Tile`] to the image mosaic.
    ///
    /// More specifically, insert the pixels of a given [`Tile`] into
    /// this image at an offset based on where that [`Tile`] belongs
    /// in the [`Mosaic`].
    pub fn add_tile(&mut self, tile: &Tile, start_coords: (u32, u32)) {
        let (start_x, start_y) = start_coords;
        self.0.copy_from(tile.img(), start_x, start_y).expect("tile should fit in mosaic");
    }
}
