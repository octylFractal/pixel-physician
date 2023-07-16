//! Build a mosaic of a particular image out of a set of smaller images.

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

#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    broken_intra_doc_links
)]

mod mosaic;
mod tiles;
mod utils;

pub use mosaic::Mosaic;
pub use utils::load_tiles;
