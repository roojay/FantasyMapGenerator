//! Fantasy Map Generator CLI (`fmg`)
//!
//! A developer-friendly command-line interface that wraps the core map-generation
//! engine and all renderer adapters.  Unlike the lower-level `map_generation`
//! binary inside `crates/core`, this tool lets you choose the output format,
//! print quick statistics and – importantly – is easy to use in automated
//! test pipelines:
//!
//! ```text
//! # Generate JSON (default)
//! fmg --seed 42 --output my_map.json
//!
//! # Generate SVG directly
//! fmg --seed 42 --format svg --output my_map.svg
//!
//! # Print map statistics to stdout (no file written)
//! fmg --seed 42 --format stats
//!
//! # Pipe JSON to another program
//! fmg --seed 42 --format json | jq '.label | length'
//! ```

use std::io::Write;

use anyhow::Result;
use clap::{Parser, ValueEnum};

use fantasy_map_core::{
    extents2d::Extents2d,
    map_generator::MapGenerator,
    rand::GlibcRand,
};
use fantasy_map_renderer_svg::{SvgAdapter, SvgConfig};

// ── CLI definition ─────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "fmg",
    about = "Fantasy Map Generator – generate procedural fantasy maps from the command line",
    long_about = "Generate procedural fantasy maps with full control over terrain, cities, and \
                  rendering format.  Supports JSON (machine-readable), SVG (vector art) and \
                  stats (developer overview)."
)]
struct Cli {
    /// Random seed (0 = deterministic default).
    #[arg(short = 's', long, default_value = "0")]
    seed: u32,

    /// Use the current wall-clock time as the random seed.
    #[arg(long, default_value = "false")]
    timeseed: bool,

    /// Output file path.  Use "-" to write to stdout.
    /// For `--format stats` the path is ignored and output always goes to stdout.
    #[arg(short = 'o', long, default_value = "-")]
    output: String,

    /// Output format.
    #[arg(short = 'f', long, default_value = "json", value_enum)]
    format: OutputFormat,

    /// Image width in pixels (used for SVG viewport and normalisation).
    #[arg(long, default_value = "1920")]
    width: u32,

    /// Image height in pixels.
    #[arg(long, default_value = "1080")]
    height: u32,

    /// Poisson-disc sampling resolution (smaller = denser grid).
    #[arg(short = 'r', long, default_value = "0.08")]
    resolution: f64,

    /// Number of cities (-1 = random 3–7).
    #[arg(short = 'c', long, default_value = "-1")]
    cities: i32,

    /// Number of towns (-1 = random 8–25).
    #[arg(short = 't', long, default_value = "-1")]
    towns: i32,

    /// Erosion amount per step (-1 = random 0.2–0.35).
    #[arg(short = 'e', long, default_value = "-1.0")]
    erosion_amount: f64,

    /// Number of erosion iterations.
    #[arg(long, default_value = "3")]
    erosion_steps: i32,

    /// Drawing scale factor (affects label font sizes).
    #[arg(long, default_value = "1.0")]
    draw_scale: f64,

    /// Disable slope rendering.
    #[arg(long)]
    no_slopes: bool,

    /// Disable river rendering.
    #[arg(long)]
    no_rivers: bool,

    /// Disable contour lines.
    #[arg(long)]
    no_contour: bool,

    /// Disable territory borders.
    #[arg(long)]
    no_borders: bool,

    /// Disable city placement.
    #[arg(long)]
    no_cities: bool,

    /// Disable town placement.
    #[arg(long)]
    no_towns: bool,

    /// Disable all labels.
    #[arg(long)]
    no_labels: bool,

    /// Disable area labels only.
    #[arg(long)]
    no_area_labels: bool,

    /// SVG coordinate decimal precision (only used with --format svg).
    #[arg(long, default_value = "2")]
    svg_precision: usize,

    /// Suppress progress messages on stderr.
    #[arg(short = 'q', long)]
    quiet: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    /// Compact JSON blob suitable for consumption by the web app or other tools.
    Json,
    /// Pretty-printed JSON (human-readable).
    JsonPretty,
    /// Scalable Vector Graphics (production-quality SVG with merged paths).
    Svg,
    /// Human-readable statistics about the generated map.
    Stats,
}

// ── Entry point ────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    let seed = if cli.timeseed {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    } else {
        cli.seed
    };

    if !cli.quiet {
        eprintln!("[fmg] seed={seed}  size={}×{}  format={:?}", cli.width, cli.height, cli.format);
    }

    let map_data = generate_map(&cli, seed)?;

    let content = match cli.format {
        OutputFormat::Json => serde_json::to_string(&map_data)?,
        OutputFormat::JsonPretty => serde_json::to_string_pretty(&map_data)?,
        OutputFormat::Svg => {
            let adapter = SvgAdapter::new(SvgConfig {
                coord_precision: cli.svg_precision,
                viewport_width: cli.width,
                viewport_height: cli.height,
            });
            use fantasy_map_core::map_data::MapAdapter;
            adapter.render(&map_data)
        }
        OutputFormat::Stats => {
            format_stats(&map_data, seed, &cli)
        }
    };

    write_output(&cli.output, &content)?;

    if !cli.quiet {
        if cli.output == "-" || matches!(cli.format, OutputFormat::Stats) {
            // already printed to stdout
        } else {
            eprintln!("[fmg] wrote {} bytes → {}", content.len(), cli.output);
        }
    }

    Ok(())
}

// ── Map generation ─────────────────────────────────────────────────────────

fn generate_map(
    cli: &Cli,
    seed: u32,
) -> Result<fantasy_map_core::map_data::MapData> {
    let mut rng = GlibcRand::new(seed);
    // Advance RNG state by 1000 steps to match C++ reference implementation
    for _ in 0..1000 {
        rng.rand();
    }

    let default_h = 20.0f64;
    let aspect = cli.width as f64 / cli.height as f64;
    let extents = Extents2d::new(0.0, 0.0, aspect * default_h, default_h);

    let mut map = MapGenerator::new(extents, cli.resolution, cli.width, cli.height, rng);
    map.set_draw_scale(cli.draw_scale);

    if cli.no_slopes    { map.disable_slopes(); }
    if cli.no_rivers    { map.disable_rivers(); }
    if cli.no_contour   { map.disable_contour(); }
    if cli.no_borders   { map.disable_borders(); }
    if cli.no_cities    { map.disable_cities(); }
    if cli.no_towns     { map.disable_towns(); }
    if cli.no_labels    { map.disable_labels(); }
    if cli.no_area_labels { map.disable_area_labels(); }

    if !cli.quiet { eprintln!("[fmg] initialising Voronoi grid…"); }
    map.initialize();

    if !cli.quiet { eprintln!("[fmg] building height map…"); }
    init_heightmap(&mut map);

    let erosion_steps = cli.erosion_steps;
    let erosion_amount = if cli.erosion_amount >= 0.0 {
        cli.erosion_amount
    } else {
        map.rng_mut().random_double(0.2, 0.35)
    };
    if !cli.quiet {
        eprintln!("[fmg] eroding: amount={erosion_amount:.3}  steps={erosion_steps}");
    }
    erode(&mut map, erosion_amount, erosion_steps);

    let num_cities = if cli.cities >= 0 { cli.cities } else { map.rng_mut().random_range(3, 7) };
    let num_towns  = if cli.towns  >= 0 { cli.towns  } else { map.rng_mut().random_range(8, 25) };

    // Load city/town names from bundled city data inside the core crate
    let num_labels = (2 * num_cities + num_towns) as usize;
    let names = load_city_names(num_labels, map.rng_mut());

    if !cli.quiet { eprintln!("[fmg] placing {num_cities} cities, {num_towns} towns…"); }
    let mut idx = names.len();
    for _ in 0..num_cities {
        if idx >= 2 {
            idx -= 2;
            map.add_city(names[idx + 1].clone(), names[idx].to_uppercase());
        }
    }
    for _ in 0..num_towns {
        if idx >= 1 {
            idx -= 1;
            map.add_town(names[idx].clone());
        }
    }

    if !cli.quiet { eprintln!("[fmg] computing draw data…"); }
    Ok(map.get_map_data())
}

// ── Terrain helpers ────────────────────────────────────────────────────────

fn init_heightmap(map: &mut MapGenerator) {
    let pad = 5.0f64;
    let ext = map.get_extents();
    let expanded = Extents2d::new(
        ext.minx - pad, ext.miny - pad,
        ext.maxx + pad, ext.maxy + pad,
    );

    let n = map.rng_mut().random_double(100.0, 250.0) as i32;
    for _ in 0..n {
        // Double-call mirrors the C++ RNG state advancement
        let _discard = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let py = map.rng_mut().random_double(expanded.miny, expanded.maxy);
        let r  = map.rng_mut().random_double(1.0, 8.0);
        let s  = map.rng_mut().random_double(0.5, 1.5);
        if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
            map.add_hill(px, py, r, s);
        } else {
            map.add_cone(px, py, r, s);
        }
    }

    if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
        let _d = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let py = map.rng_mut().random_double(expanded.miny, expanded.maxy);
        let r  = map.rng_mut().random_double(6.0, 12.0);
        let s  = map.rng_mut().random_double(1.0, 3.0);
        map.add_cone(px, py, r, s);
    }

    if map.rng_mut().random_double(0.0, 1.0) > 0.1 {
        let angle = map.rng_mut().random_double(0.0, 2.0 * std::f64::consts::PI);
        let ext2  = map.get_extents();
        let _d = map.rng_mut().random_double(ext2.minx, ext2.maxx);
        let lx = map.rng_mut().random_double(ext2.minx, ext2.maxx);
        let ly = map.rng_mut().random_double(ext2.miny, ext2.maxy);
        let sw = map.rng_mut().random_double(0.5, 5.0);
        let s  = map.rng_mut().random_double(2.0, 3.0);
        map.add_slope(lx, ly, angle.sin(), angle.cos(), sw, s);
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

fn load_city_names(num: usize, rng: &mut GlibcRand) -> Vec<String> {
    // The city data JSON is compiled into the *core* binary; we access it
    // here via the public constant exported from fantasy-map-core.
    let raw = fantasy_map_core::CITY_DATA;
    let json: serde_json::Value = serde_json::from_str(raw).expect("valid city JSON");
    let obj = json.as_object().unwrap();
    let countries: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
    let mut cities: Vec<String> = Vec::new();
    while cities.len() < num {
        let idx = rng.rand() as usize % countries.len();
        if let Some(arr) = json[countries[idx]].as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    cities.push(s.to_string());
                }
            }
        }
    }
    for i in (0..cities.len().saturating_sub(1)).rev() {
        let j = rng.rand() as usize % (i + 1);
        cities.swap(i, j);
    }
    cities.truncate(num);
    cities
}

// ── Stats formatter ────────────────────────────────────────────────────────

fn format_stats(
    data: &fantasy_map_core::map_data::MapData,
    seed: u32,
    _cli: &Cli,
) -> String {
    let num_slope_segments = data.slope.len() / 4;
    let total_contour_pts: usize = data.contour.iter().map(|p| p.len() / 2).sum();
    let total_river_pts: usize   = data.river.iter().map(|p| p.len() / 2).sum();
    let total_border_pts: usize  = data.territory.iter().map(|p| p.len() / 2).sum();

    format!(
        "Fantasy Map Generator — Statistics\n\
         ===================================\n\
         Seed             : {seed}\n\
         Image size       : {}×{} px\n\
         Draw scale       : {:.2}\n\
         \n\
         Contour lines    : {} paths  ({total_contour_pts} pts)\n\
         Rivers           : {} paths  ({total_river_pts} pts)\n\
         Slope segments   : {num_slope_segments}\n\
         Territory borders: {} paths  ({total_border_pts} pts)\n\
         Cities           : {}\n\
         Towns            : {}\n\
         Labels           : {}\n",
        data.image_width,
        data.image_height,
        data.draw_scale,
        data.contour.len(),
        data.river.len(),
        data.territory.len(),
        data.city.len() / 2,
        data.town.len() / 2,
        data.label.len(),
    )
}

// ── Output writer ──────────────────────────────────────────────────────────

fn write_output(path: &str, content: &str) -> Result<()> {
    if path == "-" {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(content.as_bytes())?;
    } else {
        std::fs::write(path, content.as_bytes())?;
    }
    Ok(())
}
