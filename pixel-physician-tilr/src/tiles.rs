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

use std::collections::HashMap;

use image::{DynamicImage, Rgb, RgbImage};
use rayon::iter::{IntoParallelRefIterator, IndexedParallelIterator, ParallelIterator};

/// Represents a single tile in a set; used to map
/// between pixels in the original image and images
/// in the [`TileSet`](super::TileSet).
#[derive(Debug)]
pub struct Tile {
    /// The underlying image to use for this Tile.
    img: RgbImage,
    /// The average pixel in the underlying image.
    ///
    /// This is computed only once when the tile is
    /// first created to handle the case of very large
    /// images being used as tiles and making the mapping
    /// between image pixels and Tiles very slow.
    avg: Rgb<u8>,
}

impl Tile {
    /// Compute the Euclidean distance between the color
    /// of the given pixel and the average pixel color
    /// of this Tile.
    pub fn sq_dist_to(&self, px: &Rgb<u8>) -> i32 {
        // color values for the given px
        let p_r = px.0[0] as i32;
        let p_g = px.0[1] as i32;
        let p_b = px.0[2] as i32;

        // color values for the avg px color of the tile
        let q_r = self.avg.0[0] as i32;
        let q_g = self.avg.0[1] as i32;
        let q_b = self.avg.0[2] as i32;

        // Euclidean distance
        (p_r - q_r).pow(2) + (p_g - q_g).pow(2) + (p_b - q_b).pow(2)
    }

    /// Get the underlying image for this Tile.
    pub fn img(&self) -> &RgbImage {
        &self.img
    }

    /// Get the x length.
    pub fn x_len(&self) -> u32 {
        self.img.dimensions().0
    }

    /// Get the y length.
    pub fn y_len(&self) -> u32 {
        self.img.dimensions().1
    }
}

impl From<RgbImage> for Tile {
    /// Build a [`Tile`] from an [`RgbImage`].
    fn from(img: RgbImage) -> Self {
        let avg_px_color = {
            // get total for each color in the image
            let mut tot_r = 0;
            let mut tot_g = 0;
            let mut tot_b = 0;
            for px in img.pixels() {
                tot_r += px.0[0] as usize;
                tot_g += px.0[1] as usize;
                tot_b += px.0[2] as usize;
            }

            // calculate the avg color for the image
            // TODO: to we care about integer division here?
            let num_px = img.pixels().len();
            Rgb([
                (tot_r / num_px) as u8,
                (tot_g / num_px) as u8,
                (tot_b / num_px) as u8,
            ])
        };

        Self {
            img,
            avg: avg_px_color,
        }
    }
}

/// A set of [`Tile`]s to use to build a [`Mosaic`](crate::Mosaic).
///
/// This struct provides methods to map between the pixels in the original
/// image to [`Tile`]s in order to build a [`Mosaic`](crate::Mosaic).
#[derive(Debug)]
pub struct TileSet {
    /// The [`Tile`]s in this set.
    tiles: Vec<Tile>,
}

impl TileSet {
    /// Get the x length of the tiles.
    pub fn tile_x_len(&self) -> u32 {
        self.tiles[0].x_len()
    }

    /// Get the y length of the tiles.
    pub fn tile_y_len(&self) -> u32 {
        self.tiles[0].y_len()
    }

    /// Create a mapping between pixels in the given image
    /// and [`Tile`]s in the set.
    pub fn map_to<'a>(&self, img: &'a RgbImage) -> HashMap<&'a Rgb<u8>, &Tile> {
        let mut map = HashMap::new();
        for px in img.pixels() {
            if map.contains_key(px) {
                continue; // don't duplicate closest tile calculations
            }
            map.insert(px, self.closest_tile(px));
        }

        map
    }

    /// Given a pixel, find the [`Tile`] in the set that most
    /// closely matches it.
    fn closest_tile(&self, px: &Rgb<u8>) -> &Tile {
        #[derive(Debug)]
        struct TileDist {
            idx: usize,
            dist: i32,
        }
        let min_idx = self.tiles.par_iter().enumerate()
            .map(|(i, t)| TileDist { idx: i, dist: t.sq_dist_to(px) })
            .min_by_key(|d| d.dist)
            .expect("should have at least one tile").idx;
        &self.tiles[min_idx]
    }
}

impl From<Vec<DynamicImage>> for TileSet {
    /// Build a tile set using the given images as [`Tile`]s.
    fn from(imgs: Vec<DynamicImage>) -> Self {
        let first_x = imgs[0].width();
        let first_y = imgs[0].height();

        if !imgs.iter().all(|i| i.width() == first_x && i.height() == first_y) {
            panic!("All images must be the same size!");
        }

        // build tiles from the resulting images
        Self {
            tiles: imgs.into_iter().map(|i| Tile::from(i.into_rgb8())).collect(),
        }
    }
}
