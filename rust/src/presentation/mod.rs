use crate::MapDrawData;
use serde::Serialize;
use serde_json::{json, Value};

pub mod standard_svg;
pub mod webgpu;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PresentationOutputKind {
    SvgScene,
    GpuScenePacket,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PresentationLayerMetadata {
    pub id: &'static str,
    pub label: &'static str,
    pub default_enabled: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PresentationConfigFieldType {
    Boolean,
    Integer,
    Float,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PresentationConfigFieldMetadata {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub field_type: PresentationConfigFieldType,
    pub default_value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PresentationConfigSectionMetadata {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub fields: Vec<PresentationConfigFieldMetadata>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PresentationPluginCapabilities {
    pub supports_layer_config: bool,
    pub supports_direct_svg_export: bool,
    pub requires_raster_data: bool,
    pub requires_heightmap: bool,
    pub requires_land_mask: bool,
    pub embeds_raster_images: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PresentationPluginMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub output_kind: PresentationOutputKind,
    pub capabilities: PresentationPluginCapabilities,
    pub supported_layers: Vec<PresentationLayerMetadata>,
    pub config_sections: Vec<PresentationConfigSectionMetadata>,
}

/// Render-data plugin interface.
///
/// Core map generation stays inside `MapGenerator`; plugins adapt the generated
/// `MapDrawData` into renderer-specific structured payloads for downstream
/// engines.
pub trait RenderDataPlugin {
    type Config;
    type Output;
    type Error;

    fn build(map_data: &MapDrawData, config: &Self::Config) -> Result<Self::Output, Self::Error>;
    fn metadata() -> PresentationPluginMetadata;
}

pub fn presentation_plugin_metadata() -> Vec<PresentationPluginMetadata> {
    vec![
        standard_svg::StandardSvgPlugin::metadata(),
        webgpu::WebGpuScenePlugin::metadata(),
    ]
}

pub(crate) fn default_layer_metadata() -> Vec<PresentationLayerMetadata> {
    vec![
        PresentationLayerMetadata {
            id: "slope",
            label: "Slope",
            default_enabled: true,
        },
        PresentationLayerMetadata {
            id: "river",
            label: "River",
            default_enabled: true,
        },
        PresentationLayerMetadata {
            id: "contour",
            label: "Contour",
            default_enabled: true,
        },
        PresentationLayerMetadata {
            id: "border",
            label: "Border",
            default_enabled: true,
        },
        PresentationLayerMetadata {
            id: "city",
            label: "City",
            default_enabled: true,
        },
        PresentationLayerMetadata {
            id: "town",
            label: "Town",
            default_enabled: true,
        },
        PresentationLayerMetadata {
            id: "label",
            label: "Label",
            default_enabled: true,
        },
    ]
}

pub(crate) fn default_layer_config_section() -> PresentationConfigSectionMetadata {
    PresentationConfigSectionMetadata {
        id: "layers",
        label: "Layers",
        description: "Toggle visibility for each logical overlay layer during scene construction.",
        fields: default_layer_metadata()
            .into_iter()
            .map(|layer| PresentationConfigFieldMetadata {
                key: layer.id,
                label: layer.label,
                description: "Enable or disable this layer in the generated presentation scene.",
                field_type: PresentationConfigFieldType::Boolean,
                default_value: json!(layer.default_enabled),
                min: None,
                max: None,
                step: None,
            })
            .collect(),
    }
}

pub(crate) fn integer_config_field(
    key: &'static str,
    label: &'static str,
    description: &'static str,
    default_value: i64,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
) -> PresentationConfigFieldMetadata {
    PresentationConfigFieldMetadata {
        key,
        label,
        description,
        field_type: PresentationConfigFieldType::Integer,
        default_value: json!(default_value),
        min,
        max,
        step,
    }
}

#[cfg(test)]
mod tests {
    use super::{presentation_plugin_metadata, PresentationOutputKind};

    #[test]
    fn metadata_registry_includes_core_plugins() {
        let metadata = presentation_plugin_metadata();
        let ids: Vec<_> = metadata.iter().map(|plugin| plugin.id).collect();

        assert!(ids.contains(&"standard_svg"));
        assert!(ids.contains(&"webgpu_scene"));
    }

    #[test]
    fn standard_svg_metadata_uses_svg_scene_output() {
        let metadata = presentation_plugin_metadata();
        let plugin = metadata
            .iter()
            .find(|plugin| plugin.id == "standard_svg")
            .expect("standard_svg plugin metadata should exist");

        assert_eq!(plugin.output_kind, PresentationOutputKind::SvgScene);
        assert!(plugin.capabilities.supports_direct_svg_export);
        assert_eq!(plugin.config_sections.len(), 1);
    }

}
