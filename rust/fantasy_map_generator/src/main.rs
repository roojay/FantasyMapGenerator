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
pub mod render;

use clap::Parser;
use config::Config;
use extents2d::Extents2d;
use rand::GlibcRand;
use map_generator::MapGenerator;

fn main() -> anyhow::Result<()> {
    let cfg = Config::parse();

    let seed = if cfg.timeseed {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    } else {
        cfg.seed
    };

    eprintln!("Generating map with seed value: {}", seed);

    let mut rng = GlibcRand::new(seed);
    for _ in 0..1000 {
        rng.rand();
    }

    let (img_w, img_h) = cfg.image_size();
    let default_extents_height = 20.0f64;
    let aspect = img_w as f64 / img_h as f64;
    let extents_width = aspect * default_extents_height;
    let extents = Extents2d::new(0.0, 0.0, extents_width, default_extents_height);

    let mut map = MapGenerator::new(extents, cfg.resolution, img_w, img_h, rng);
    map.set_draw_scale(cfg.draw_scale);

    if cfg.no_slopes { map.disable_slopes(); }
    if cfg.no_rivers { map.disable_rivers(); }
    if cfg.no_contour { map.disable_contour(); }
    if cfg.no_borders { map.disable_borders(); }
    if cfg.no_cities { map.disable_cities(); }
    if cfg.no_towns { map.disable_towns(); }
    if cfg.no_labels { map.disable_labels(); }
    if cfg.no_arealabels { map.disable_area_labels(); }

    eprintln!("Initializing map generator...");
    map.initialize();

    eprintln!("Initializing height map...");
    initialize_heightmap(&mut map);

    let erosion_steps = cfg.erosion_steps;
    let erosion_amount = if cfg.erosion_amount >= 0.0 {
        cfg.erosion_amount
    } else {
        map.rng_mut().random_double(0.2, 0.35)
    };
    eprintln!("Eroding height map by {} over {} iterations...", erosion_amount, erosion_steps);
    erode(&mut map, erosion_amount, erosion_steps);

    let num_cities = if cfg.cities >= 0 { cfg.cities } else { map.rng_mut().random_range(3, 7) };
    let num_towns = if cfg.towns >= 0 { cfg.towns } else { map.rng_mut().random_range(8, 25) };

    let num_labels = 2 * num_cities + num_towns;
    let label_names = get_label_names(num_labels as usize, map.rng_mut());

    eprintln!("Generating {} cities...", num_cities);
    let mut label_idx = label_names.len();
    for _ in 0..num_cities {
        if label_idx >= 2 {
            label_idx -= 2;
            let city_name = label_names[label_idx + 1].clone();
            let territory_name = label_names[label_idx].to_uppercase();
            map.add_city(city_name, territory_name);
        }
    }

    eprintln!("Generating {} towns...", num_towns);
    for _ in 0..num_towns {
        if label_idx >= 1 {
            label_idx -= 1;
            let town_name = label_names[label_idx].clone();
            map.add_town(town_name);
        }
    }

    eprintln!("Generating map draw data...");
    let draw_data = map.get_draw_data();

    let outfile = if cfg.output.ends_with(".json") {
        cfg.output.clone()
    } else {
        format!("{}.json", cfg.output)
    };

    std::fs::write(&outfile, draw_data.as_bytes())?;
    eprintln!("Wrote map draw data to file: {}", outfile);

    Ok(())
}

// NOTE: The double-assignment pattern below (px assigned twice) intentionally
// mirrors the C++ source code behavior where randomDouble() is called twice for x
// but only the second value is used. This preserves exact RNG state compatibility.

fn initialize_heightmap(map: &mut MapGenerator) {
    let pad = 5.0f64;
    let extents = map.get_extents();
    let expanded = Extents2d::new(
        extents.minx - pad, extents.miny - pad,
        extents.maxx + pad, extents.maxy + pad,
    );

    let n = map.rng_mut().random_double(100.0, 250.0) as i32;
    for _ in 0..n {
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let py = map.rng_mut().random_double(expanded.miny, expanded.maxy);
        let r = map.rng_mut().random_double(1.0, 8.0);
        let strength = map.rng_mut().random_double(0.5, 1.5);
        if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
            map.add_hill(px, py, r, strength);
        } else {
            map.add_cone(px, py, r, strength);
        }
    }

    if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let py = map.rng_mut().random_double(expanded.miny, expanded.maxy);
        let r = map.rng_mut().random_double(6.0, 12.0);
        let strength = map.rng_mut().random_double(1.0, 3.0);
        map.add_cone(px, py, r, strength);
    }

    if map.rng_mut().random_double(0.0, 1.0) > 0.1 {
        let angle = map.rng_mut().random_double(0.0, 2.0 * std::f64::consts::PI);
        let dir_x = angle.sin();
        let dir_y = angle.cos();
        let lx = map.rng_mut().random_double(extents.minx, extents.maxx);
        let lx = map.rng_mut().random_double(extents.minx, extents.maxx);
        let ly = map.rng_mut().random_double(extents.miny, extents.maxy);
        let slope_width = map.rng_mut().random_double(0.5, 5.0);
        let strength = map.rng_mut().random_double(2.0, 3.0);
        map.add_slope(lx, ly, dir_x, dir_y, slope_width, strength);
    }

    if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
        map.normalize();
    } else {
        map.round();
    }

    if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
        map.relax();
    }
}

fn erode(map: &mut MapGenerator, amount: f64, iterations: i32) {
    for _ in 0..iterations {
        map.erode(amount / iterations as f64);
    }
    map.set_sea_level_to_median();
}

fn get_label_names(num: usize, rng: &mut GlibcRand) -> Vec<String> {
    let city_data = include_str!("citydata/countrycities.json");
    let json: serde_json::Value = serde_json::from_str(city_data).expect("valid JSON");
    let obj = json.as_object().unwrap();
    let countries: Vec<String> = obj.keys().cloned().collect();
    let mut cities: Vec<String> = Vec::new();

    while cities.len() < num {
        let rand_idx = rng.rand() as usize % countries.len();
        let country = &countries[rand_idx];
        if let Some(arr) = json[country].as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    cities.push(s.to_string());
                }
            }
        }
    }

    // Shuffle (matching C++ implementation)
    for i in (0..cities.len().saturating_sub(1)).rev() {
        let j = rng.rand() as usize % (i + 1);
        cities.swap(i, j);
    }

    cities.truncate(num);
    cities
}
