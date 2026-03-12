use crate::presentation::standard_svg::{
    render_standard_svg_scene, StandardSvgLayers, StandardSvgPlugin,
};
use crate::presentation::RenderDataPlugin;
use crate::MapDrawData;

pub fn build_map_svg(map_json: &str, layers_json: &str) -> Result<String, String> {
    let map_data: MapDrawData =
        serde_json::from_str(map_json).map_err(|err| format!("Invalid map JSON: {err}"))?;
    let layers = if layers_json.trim().is_empty() {
        StandardSvgLayers::default()
    } else {
        serde_json::from_str(layers_json).map_err(|err| format!("Invalid layers JSON: {err}"))?
    };

    let scene = StandardSvgPlugin::build(&map_data, &layers)?;
    Ok(render_standard_svg_scene(&scene))
}

#[cfg(test)]
mod tests {
    use super::build_map_svg;

    #[test]
    fn standard_svg_contains_core_layers() {
        let map_json = serde_json::json!({
            "image_width": 128,
            "image_height": 64,
            "draw_scale": 1.0,
            "slope": [0.1, 0.1, 0.2, 0.2],
            "river": [[0.2, 0.2, 0.4, 0.4]],
            "contour": [[0.1, 0.8, 0.9, 0.8]],
            "territory": [[0.15, 0.15, 0.85, 0.15]],
            "city": [0.25, 0.25],
            "town": [0.75, 0.25],
            "label": [{
                "text": "Testoria",
                "fontface": "Times New Roman",
                "fontsize": 14,
                "position": [0.3, 0.4],
                "score": 1.0,
                "extents": [0.0, 0.0, 0.0, 0.0],
                "charextents": []
            }]
        })
        .to_string();
        let layers_json = serde_json::json!({
            "slope": true,
            "river": true,
            "contour": true,
            "border": true,
            "city": true,
            "town": true,
            "label": true
        })
        .to_string();

        let svg = build_map_svg(&map_json, &layers_json).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("<text"));
    }
}
