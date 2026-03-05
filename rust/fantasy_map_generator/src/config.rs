use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "map_generation", about = "Fantasy Map Generator")]
pub struct Config {
    /// Random seed
    #[arg(long, default_value = "0")]
    pub seed: u32,

    /// Use current time as seed
    #[arg(long, default_value = "false")]
    pub timeseed: bool,

    /// Output file (without extension)
    #[arg(long, short = 'o', default_value = "output")]
    pub output: String,

    /// Resolution (poisson disc sampling distance)
    #[arg(long, default_value = "0.45")]
    pub resolution: f64,

    /// Erosion amount (-1 for random)
    #[arg(long, default_value = "-1.0")]
    pub erosion_amount: f64,

    /// Erosion iterations
    #[arg(long, default_value = "5")]
    pub erosion_steps: i32,

    /// Number of cities (-1 for random)
    #[arg(long, default_value = "-1")]
    pub cities: i32,

    /// Number of towns (-1 for random)
    #[arg(long, default_value = "-1")]
    pub towns: i32,

    /// Image size (e.g. "1920x1080")
    #[arg(long, default_value = "1920x1080")]
    pub size: String,

    /// Draw scale
    #[arg(long, default_value = "1.0")]
    pub draw_scale: f64,

    /// Disable slopes
    #[arg(long, default_value = "false")]
    pub no_slopes: bool,

    /// Disable rivers
    #[arg(long, default_value = "false")]
    pub no_rivers: bool,

    /// Disable contour
    #[arg(long, default_value = "false")]
    pub no_contour: bool,

    /// Disable territory borders
    #[arg(long, default_value = "false")]
    pub no_borders: bool,

    /// Disable cities
    #[arg(long, default_value = "false")]
    pub no_cities: bool,

    /// Disable towns
    #[arg(long, default_value = "false")]
    pub no_towns: bool,

    /// Disable labels
    #[arg(long, default_value = "false")]
    pub no_labels: bool,

    /// Disable area labels
    #[arg(long, default_value = "false")]
    pub no_arealabels: bool,
}

impl Config {
    pub fn image_size(&self) -> (u32, u32) {
        let parts: Vec<&str> = self.size.split('x').collect();
        if parts.len() == 2 {
            let w = parts[0].parse().unwrap_or(1920);
            let h = parts[1].parse().unwrap_or(1080);
            (w, h)
        } else {
            (1920, 1080)
        }
    }
}
