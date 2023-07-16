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

use image::io::Reader as ImageReader;
use image::DynamicImage;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Load all images at the given `path` to use as tiles in the [`Mosaic`][crate::Mosaic]
pub fn load_tiles(path: &Path) -> Result<Vec<DynamicImage>, Box<dyn Error>> {
    if !path.is_dir() {
        return Err(format!("Path must be a directory: {}", path.display()).into());
    }

    let mut tiles = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let tile = load(&path)?;
            tiles.push(tile);
        }
    }

    Ok(tiles)
}

/// Load a single image to use as a tile in the [`Mosaic`][crate::Mosaic]
fn load(tile: &Path) -> Result<DynamicImage, Box<dyn Error>> {
    Ok(ImageReader::open(tile)?.decode()?)
}
