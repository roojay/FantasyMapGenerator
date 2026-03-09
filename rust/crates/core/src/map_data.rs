//! Core data types and traits for fantasy map generation.
//!
//! `MapData` represents the raw output of the map generation algorithm.
//! `MapAdapter` is the plugin trait that all renderers must implement.

use serde::{Deserialize, Serialize};

/// A single text label with layout information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelData {
    /// The text to display.
    pub text: String,
    /// Font family name.
    pub fontface: String,
    /// Font size in points.
    pub fontsize: i32,
    /// Normalised [0,1] position [x, y].
    pub position: [f64; 2],
    /// Normalised [0,1] bounding box [minx, miny, maxx, maxy].
    pub extents: [f64; 4],
    /// Per-character bounding boxes as flat [minx, miny, maxx, maxy, ...].
    pub char_extents: Vec<f64>,
    /// Placement quality score (higher is better).
    pub score: f64,
}

/// Raw output produced by the core map-generation algorithm.
///
/// All coordinates are normalised to `[0.0, 1.0]` in both axes.
/// Paths are represented as flat `Vec<f64>` sequences of `[x, y, x, y, …]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    /// Image width in pixels (informational).
    pub image_width: u32,
    /// Image height in pixels (informational).
    pub image_height: u32,
    /// Coordinate-to-pixel scale factor.
    pub draw_scale: f64,
    /// Contour polylines: each inner `Vec` is `[x0,y0, x1,y1, …]`.
    pub contour: Vec<Vec<f64>>,
    /// River polylines: each inner `Vec` is `[x0,y0, x1,y1, …]`.
    pub river: Vec<Vec<f64>>,
    /// Slope line-segments as `[x0,y0, x1,y1, …]` (flat).
    pub slope: Vec<f64>,
    /// City positions as `[x0,y0, x1,y1, …]` (flat).
    pub city: Vec<f64>,
    /// Town positions as `[x0,y0, x1,y1, …]` (flat).
    pub town: Vec<f64>,
    /// Territory border polylines: each inner `Vec` is `[x0,y0, x1,y1, …]`.
    pub territory: Vec<Vec<f64>>,
    /// Text labels.
    pub label: Vec<LabelData>,
}

/// Plugin contract: convert `MapData` into a specific output format.
///
/// Implement this trait in a renderer crate to add a new output format
/// (SVG string, WASM memory layout, PNG bytes, etc.).
pub trait MapAdapter {
    /// The final product produced by this adapter.
    type Output;

    /// Transform raw `MapData` into `Self::Output`.
    fn render(&self, data: &MapData) -> Self::Output;
}
