//! High-performance SVG renderer for Fantasy Map Generator.
//!
//! Converts [`MapData`] to a production-quality SVG string with:
//! - Layer-based `<g>` grouping
//! - Coordinate precision control (2 decimal places)
//! - Path merging for adjacent same-type polylines into single `<path>` elements

use fantasy_map_core::map_data::{MapAdapter, MapData};

/// Configuration options for the SVG renderer.
#[derive(Debug, Clone)]
pub struct SvgConfig {
    /// Number of decimal places for coordinates (default: 2).
    pub coord_precision: usize,
    /// Width of the SVG viewport in pixels (default: 1920).
    pub viewport_width: u32,
    /// Height of the SVG viewport in pixels (default: 1080).
    pub viewport_height: u32,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            coord_precision: 2,
            viewport_width: 1920,
            viewport_height: 1080,
        }
    }
}

/// SVG rendering adapter.
pub struct SvgAdapter {
    pub config: SvgConfig,
}

impl SvgAdapter {
    pub fn new(config: SvgConfig) -> Self {
        Self { config }
    }

    fn fmt_coord(&self, v: f64, scale: f64) -> String {
        let scaled = v * scale;
        format!("{:.prec$}", scaled, prec = self.config.coord_precision)
    }

    /// Build a polyline path string: "M x0,y0 L x1,y1 …"
    fn polyline_to_path_data(&self, coords: &[f64], w: f64, h: f64) -> String {
        if coords.len() < 4 {
            return String::new();
        }
        let mut parts = Vec::with_capacity(coords.len() / 2 + 1);
        let mut i = 0;
        while i + 1 < coords.len() {
            let x = self.fmt_coord(coords[i], w);
            let y = self.fmt_coord(coords[i + 1], h);
            if i == 0 {
                parts.push(format!("M{},{}", x, y));
            } else {
                parts.push(format!("L{},{}", x, y));
            }
            i += 2;
        }
        parts.join(" ")
    }

    /// Merge multiple polylines that share the same style into a single `<path>` element.
    fn merged_path_element(
        &self,
        polylines: &[Vec<f64>],
        w: f64,
        h: f64,
        attrs: &str,
    ) -> String {
        let combined: String = polylines
            .iter()
            .filter(|p| p.len() >= 4)
            .map(|p| self.polyline_to_path_data(p, w, h))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if combined.is_empty() {
            return String::new();
        }
        format!("<path {} d=\"{}\"/>", attrs, combined)
    }
}

impl MapAdapter for SvgAdapter {
    type Output = String;

    fn render(&self, data: &MapData) -> String {
        let w = self.config.viewport_width as f64;
        let h = self.config.viewport_height as f64;
        let prec = self.config.coord_precision;

        let mut svg = String::with_capacity(1 << 20); // 1 MB pre-alloc

        // ── SVG header ──────────────────────────────────────────────────────
        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">",
        ));

        // ── Background ──────────────────────────────────────────────────────
        svg.push_str(&format!("<rect width=\"{w}\" height=\"{h}\" fill=\"#c8e8f8\"/>"));

        // ── Layer: contour ──────────────────────────────────────────────────
        svg.push_str("<g id=\"contour\" stroke=\"#8899aa\" stroke-width=\"0.5\" fill=\"none\">");
        let merged_contour = self.merged_path_element(
            &data.contour, w, h,
            "stroke=\"#8899aa\" stroke-width=\"0.5\" fill=\"none\"",
        );
        svg.push_str(&merged_contour);
        svg.push_str("</g>");

        // ── Layer: rivers ───────────────────────────────────────────────────
        svg.push_str("<g id=\"rivers\" stroke=\"#4488cc\" stroke-width=\"1.0\" fill=\"none\">");
        let merged_river = self.merged_path_element(
            &data.river, w, h,
            "stroke=\"#4488cc\" stroke-width=\"1.0\" fill=\"none\"",
        );
        svg.push_str(&merged_river);
        svg.push_str("</g>");

        // ── Layer: slopes ────────────────────────────────────────────────────
        svg.push_str("<g id=\"slopes\" stroke=\"#555566\" stroke-width=\"0.7\" fill=\"none\">");
        let mut si = 0;
        while si + 3 < data.slope.len() {
            let x1 = format!("{:.prec$}", data.slope[si] * w, prec = prec);
            let y1 = format!("{:.prec$}", data.slope[si + 1] * h, prec = prec);
            let x2 = format!("{:.prec$}", data.slope[si + 2] * w, prec = prec);
            let y2 = format!("{:.prec$}", data.slope[si + 3] * h, prec = prec);
            svg.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                x1, y1, x2, y2
            ));
            si += 4;
        }
        svg.push_str("</g>");

        // ── Layer: territory borders ─────────────────────────────────────────
        svg.push_str("<g id=\"territory\" stroke=\"#cc4444\" stroke-width=\"1.0\" fill=\"none\" stroke-dasharray=\"4,3\">");
        let merged_territory = self.merged_path_element(
            &data.territory, w, h,
            "stroke=\"#cc4444\" stroke-width=\"1.0\" fill=\"none\" stroke-dasharray=\"4,3\"",
        );
        svg.push_str(&merged_territory);
        svg.push_str("</g>");

        // ── Layer: cities ─────────────────────────────────────────────────────
        svg.push_str("<g id=\"cities\" fill=\"#222233\">");
        let mut ci = 0;
        while ci + 1 < data.city.len() {
            let cx = format!("{:.prec$}", data.city[ci] * w, prec = prec);
            let cy = format!("{:.prec$}", data.city[ci + 1] * h, prec = prec);
            svg.push_str(&format!("<circle cx=\"{}\" cy=\"{}\" r=\"4\"/>", cx, cy));
            ci += 2;
        }
        svg.push_str("</g>");

        // ── Layer: towns ──────────────────────────────────────────────────────
        svg.push_str("<g id=\"towns\" fill=\"#445566\">");
        let mut ti = 0;
        while ti + 1 < data.town.len() {
            let cx = format!("{:.prec$}", data.town[ti] * w, prec = prec);
            let cy = format!("{:.prec$}", data.town[ti + 1] * h, prec = prec);
            svg.push_str(&format!("<circle cx=\"{}\" cy=\"{}\" r=\"2.5\"/>", cx, cy));
            ti += 2;
        }
        svg.push_str("</g>");

        // ── Layer: labels ─────────────────────────────────────────────────────
        svg.push_str("<g id=\"labels\" font-family=\"serif\">");
        for lbl in &data.label {
            let lx = format!("{:.prec$}", lbl.position[0] * w, prec = prec);
            let ly = format!("{:.prec$}", lbl.position[1] * h, prec = prec);
            // Escape XML special characters in text
            let escaped: String = lbl.text.chars().map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '"' => "&quot;".to_string(),
                c   => c.to_string(),
            }).collect();
            svg.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"#111122\">{}</text>",
                lx, ly,
                (lbl.fontsize as f64 * data.draw_scale) as i32,
                escaped
            ));
        }
        svg.push_str("</g>");

        svg.push_str("</svg>");
        svg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fantasy_map_core::map_data::{LabelData, MapData};

    fn sample_data() -> MapData {
        MapData {
            image_width: 800,
            image_height: 600,
            draw_scale: 1.0,
            contour: vec![vec![0.1, 0.1, 0.2, 0.2, 0.3, 0.1]],
            river: vec![vec![0.4, 0.5, 0.5, 0.6]],
            slope: vec![0.1, 0.2, 0.15, 0.25],
            city: vec![0.5, 0.5],
            town: vec![0.3, 0.3],
            territory: vec![vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0]],
            label: vec![LabelData {
                text: "Test City".to_string(),
                fontface: "serif".to_string(),
                fontsize: 14,
                position: [0.5, 0.5],
                extents: [0.4, 0.45, 0.6, 0.55],
                char_extents: vec![],
                score: 1.0,
            }],
        }
    }

    #[test]
    fn test_svg_adapter_produces_valid_svg() {
        let adapter = SvgAdapter::new(SvgConfig {
            viewport_width: 800,
            viewport_height: 600,
            ..Default::default()
        });
        let svg = adapter.render(&sample_data());
        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.ends_with("</svg>"), "SVG must end with </svg>");
        assert!(svg.contains(r#"id="contour""#), "SVG must have contour layer");
        assert!(svg.contains(r#"id="rivers""#), "SVG must have rivers layer");
        assert!(svg.contains(r#"id="slopes""#), "SVG must have slopes layer");
        assert!(svg.contains(r#"id="territory""#), "SVG must have territory layer");
        assert!(svg.contains(r#"id="cities""#), "SVG must have cities layer");
        assert!(svg.contains(r#"id="towns""#), "SVG must have towns layer");
        assert!(svg.contains(r#"id="labels""#), "SVG must have labels layer");
        assert!(svg.contains("Test City"), "SVG must contain label text");
    }

    #[test]
    fn test_svg_path_merging() {
        let adapter = SvgAdapter::new(SvgConfig::default());
        let data = MapData {
            contour: vec![
                vec![0.1, 0.1, 0.2, 0.2],
                vec![0.3, 0.3, 0.4, 0.4],
                vec![0.5, 0.5, 0.6, 0.6],
            ],
            ..sample_data()
        };
        let svg = adapter.render(&data);
        // Multiple contour polylines should be merged into one <path> element
        let path_count = svg.matches("<path").count();
        // Territory and contour each produce 1 merged path → 2 total
        assert!(path_count <= 3, "Path merging should reduce <path> count; got {}", path_count);
    }

    #[test]
    fn test_svg_precision() {
        let adapter = SvgAdapter::new(SvgConfig {
            coord_precision: 1,
            viewport_width: 1000,
            viewport_height: 1000,
        });
        let data = MapData {
            city: vec![0.1234, 0.5678],
            ..sample_data()
        };
        let svg = adapter.render(&data);
        // With precision=1, 0.1234*1000 = 123.4 → "123.4"
        assert!(svg.contains("123.4"), "Precision 1 should give 1 decimal place");
    }
}
