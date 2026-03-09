use std::process::Command;
use std::path::PathBuf;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    // Walk up until we find the target directory
    loop {
        if path.ends_with("target") {
            break;
        }
        if !path.pop() {
            panic!("Could not find target directory");
        }
    }
    path.join("debug").join("map_generation")
}

#[test]
fn test_output_is_valid_json() {
    let outdir = std::env::temp_dir();
    let outfile = outdir.join("test_map_rust_0.json");

    let status = Command::new(binary_path())
        .args(["--seed", "0", "--output", outfile.to_str().unwrap()])
        .status()
        .expect("Failed to run map_generation binary");

    assert!(status.success(), "map_generation binary failed");

    let content = std::fs::read_to_string(&outfile).expect("Output file missing");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Output is not valid JSON");

    assert_eq!(v["image_width"].as_u64().unwrap(), 1920);
    assert_eq!(v["image_height"].as_u64().unwrap(), 1080);
    assert!(v["draw_scale"].is_number());
    assert!(v["contour"].is_array());
    assert!(v["river"].is_array());
    assert!(v["slope"].is_array());
    assert!(v["city"].is_array());
    assert!(v["town"].is_array());
    assert!(v["territory"].is_array());
    assert!(v["label"].is_array());

    let _ = std::fs::remove_file(&outfile);
}
