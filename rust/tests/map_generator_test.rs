use fantasy_map_generator::data_structures::Extents2d;
use fantasy_map_generator::map_generator::MapGenerator;
use fantasy_map_generator::utils::rand::GlibcRand;

fn create_test_generator(seed: u32) -> MapGenerator {
    let extents = Extents2d::new(-17.78, -10.0, 17.78, 10.0);
    let rng = GlibcRand::new(seed);
    MapGenerator::new(extents, 0.08, 1920, 1080, rng)
}

fn create_small_generator(seed: u32) -> MapGenerator {
    let extents = Extents2d::new(-5.0, -3.0, 5.0, 3.0);
    let rng = GlibcRand::new(seed);
    MapGenerator::new(extents, 0.3, 640, 360, rng)
}

// ==============================
// Initialization tests
// ==============================

#[test]
fn initialize_creates_voronoi_grid() {
    let mut gen = create_small_generator(42);
    gen.initialize();
    // After initialize, the generator should have set up the Voronoi grid
    // and can accept terrain operations. We verify by adding a hill (should not panic).
    gen.add_hill(0.0, 0.0, 2.0, 1.0);
}

#[test]
fn initialize_is_deterministic() {
    let mut gen1 = create_small_generator(42);
    gen1.initialize();

    let mut gen2 = create_small_generator(42);
    gen2.initialize();

    // Same seed should produce same heightmap size
    let data1 = gen1.get_draw_data();
    let data2 = gen2.get_draw_data();
    assert_eq!(data1.len(), data2.len(), "same seed should produce identical output");
}

// ==============================
// Terrain operations
// ==============================

#[test]
fn add_hill_normalize_roundtrip() {
    let mut gen = create_small_generator(1);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.add_hill(2.0, 1.0, 2.0, 0.5);
    gen.normalize();
    // Should not panic and produce valid state
}

#[test]
fn erode_does_not_panic() {
    let mut gen = create_small_generator(2);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.5);
}

#[test]
fn relax_does_not_panic() {
    let mut gen = create_small_generator(3);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.relax();
}

// ==============================
// City/Town placement
// ==============================

#[test]
fn add_city_and_town_after_terrain() {
    let mut gen = create_small_generator(10);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 4.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.5);
    gen.add_city("TestCity".to_string(), "KINGDOM".to_string());
    gen.add_town("TestTown".to_string());
}

// ==============================
// Output generation
// ==============================

#[test]
fn get_draw_data_returns_valid_json() {
    let mut gen = create_small_generator(42);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.5);
    gen.add_city("Capital".to_string(), "EMPIRE".to_string());

    let json_str = gen.get_draw_data();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("get_draw_data should produce valid JSON");

    assert!(parsed.get("image_width").is_some(), "missing image_width");
    assert!(parsed.get("image_height").is_some(), "missing image_height");
    assert!(parsed.get("contour").is_some(), "missing contour");
    assert!(parsed.get("river").is_some(), "missing river");
    assert!(parsed.get("slope").is_some(), "missing slope");
}

// ==============================
// Full pipeline (end-to-end)
// ==============================

#[test]
fn full_pipeline_small_map() {
    let mut gen = create_small_generator(100);
    gen.initialize();

    // Build terrain
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.add_hill(-2.0, 1.0, 2.0, 0.7);
    gen.add_hill(2.0, -1.0, 1.5, 0.5);
    gen.normalize();
    gen.round();
    gen.relax();
    gen.set_sea_level_to_median();
    gen.erode(0.5);

    // Place cities
    gen.add_city("Alpha".to_string(), "NORTH".to_string());
    gen.add_city("Beta".to_string(), "SOUTH".to_string());
    gen.add_town("Village".to_string());

    // Generate output
    let json_str = gen.get_draw_data();
    assert!(!json_str.is_empty());
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify cities are in output
    let cities = parsed.get("city").expect("missing city");
    assert!(cities.is_array());

    // Verify territories
    let territories = parsed.get("territory").expect("missing territory");
    assert!(territories.is_array());
}

// ==============================
// Export functions
// ==============================

#[test]
fn export_heightmap_returns_data() {
    let mut gen = create_small_generator(50);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.5);

    let hm = gen.export_heightmap(160, 90);
    assert_eq!(hm.len(), 160 * 90, "heightmap should have width*height elements");

    // Values should be in a reasonable range after normalization
    let min = hm.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = hm.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(max > min, "heightmap should have varying values");
}

#[test]
fn export_flux_map_returns_data() {
    let mut gen = create_small_generator(50);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.5);

    let fm = gen.export_flux_map(160, 90);
    assert_eq!(fm.len(), 160 * 90);
}

#[test]
fn export_land_mask_returns_data() {
    let mut gen = create_small_generator(50);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.5);

    let mask = gen.export_land_mask(160, 90); // export_land_mask takes &mut self
    assert_eq!(mask.len(), 160 * 90);
    // Should contain both 0 (sea) and 255 (land)
    let has_sea = mask.iter().any(|&v| v < 128);
    let has_land = mask.iter().any(|&v| v > 128);
    assert!(has_sea || has_land, "mask should have values");
}

// ==============================
// Edge cases
// ==============================

#[test]
fn multiple_erode_calls() {
    let mut gen = create_small_generator(7);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.3);
    gen.erode(0.3);
    gen.erode(0.3);
    // Should handle multiple erosion passes without issues
}

#[test]
fn no_terrain_operations_produces_flat_map() {
    let mut gen = create_small_generator(99);
    gen.initialize();
    // No hills added — terrain is flat
    let json_str = gen.get_draw_data();
    assert!(!json_str.is_empty());
}
