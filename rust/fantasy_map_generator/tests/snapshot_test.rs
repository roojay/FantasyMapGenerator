use std::process::Command;

#[test]
fn test_output_is_valid_json() {
    let binary = env!("CARGO_BIN_EXE_map_generation");
    let output = Command::new(binary)
        .args(["--seed", "0", "--output", "/tmp/test_map_0"])
        .output()
        .expect("failed to run binary");

    assert!(output.status.success(), "Binary failed: {}", String::from_utf8_lossy(&output.stderr));

    let json_str = std::fs::read_to_string("/tmp/test_map_0.json")
        .expect("output file not found");

    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .expect("output is not valid JSON");

    assert!(parsed.get("image_width").is_some(), "missing image_width");
    assert!(parsed.get("image_height").is_some(), "missing image_height");
    assert!(parsed.get("draw_scale").is_some(), "missing draw_scale");
    assert!(parsed.get("contour").is_some(), "missing contour");
    assert!(parsed.get("river").is_some(), "missing river");
    assert!(parsed.get("slope").is_some(), "missing slope");
    assert!(parsed.get("city").is_some(), "missing city");
    assert!(parsed.get("town").is_some(), "missing town");
    assert!(parsed.get("territory").is_some(), "missing territory");
    assert!(parsed.get("label").is_some(), "missing label");

    assert_eq!(parsed["image_width"].as_u64().unwrap(), 1920);
    assert_eq!(parsed["image_height"].as_u64().unwrap(), 1080);

    let _ = std::fs::remove_file("/tmp/test_map_0.json"); // ignore if already deleted
}
