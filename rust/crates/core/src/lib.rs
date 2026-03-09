//! Core map generation library for Fantasy Map Generator.
//!
//! This crate contains the pure map-generation algorithm that produces
//! a [`MapData`] value, and the [`MapAdapter`] trait that all renderer
//! plug-ins must implement.

#![allow(dead_code, unused_imports, unused_variables)]

pub mod config;
pub mod rand;
pub mod geometry;
pub mod extents2d;
pub mod dcel;
pub mod poisson_disc;
pub mod delaunay;
pub mod voronoi;
pub mod vertex_map;
pub mod node_map;
pub mod font_face;
pub mod spatial_point_grid;
pub mod map_generator;
pub mod map_data;
pub mod render;

pub use map_data::{LabelData, MapAdapter, MapData};
pub use map_generator::MapGenerator;
pub use extents2d::Extents2d;
pub use rand::GlibcRand;

/// City name data compiled into the library at build time.
///
/// A JSON object mapping country names to arrays of city names.
/// Used by both the `map_generation` binary and the `fmg` CLI.
pub const CITY_DATA: &str = include_str!("citydata/countrycities.json");
