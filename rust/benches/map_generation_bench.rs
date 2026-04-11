use criterion::{criterion_group, criterion_main, Criterion};
use fantasy_map_generator::algorithms::{delaunay, poisson_disc, voronoi};
use fantasy_map_generator::data_structures::Extents2d;
use fantasy_map_generator::map_generator::MapGenerator;
use fantasy_map_generator::utils::rand::GlibcRand;

// ==============================
// Algorithm benchmarks
// ==============================

fn bench_poisson_disc_sampling(c: &mut Criterion) {
    c.bench_function("poisson_disc r=0.08 (35x20)", |b| {
        b.iter(|| {
            let mut rng = GlibcRand::new(42);
            let bounds = Extents2d::new(-17.78, -10.0, 17.78, 10.0);
            poisson_disc::generate_samples(&mut rng, bounds, 0.08, 25)
        })
    });

    c.bench_function("poisson_disc r=0.3 (10x6)", |b| {
        b.iter(|| {
            let mut rng = GlibcRand::new(42);
            let bounds = Extents2d::new(-5.0, -3.0, 5.0, 3.0);
            poisson_disc::generate_samples(&mut rng, bounds, 0.3, 25)
        })
    });
}

fn bench_delaunay_triangulation(c: &mut Criterion) {
    // Pre-generate points for consistent benchmark
    let mut rng = GlibcRand::new(42);
    let bounds = Extents2d::new(-17.78, -10.0, 17.78, 10.0);
    let reference_points = poisson_disc::generate_samples(&mut rng, bounds, 0.08, 25);

    c.bench_function(
        &format!("delaunay triangulate ({} points)", reference_points.len()),
        |b| {
            b.iter(|| {
                let mut pts = reference_points.clone();
                delaunay::triangulate(&mut pts)
            })
        },
    );
}

fn bench_voronoi_construction(c: &mut Criterion) {
    let mut rng = GlibcRand::new(42);
    let bounds = Extents2d::new(-17.78, -10.0, 17.78, 10.0);
    let mut points = poisson_disc::generate_samples(&mut rng, bounds, 0.08, 25);
    let delaunay_dcel = delaunay::triangulate(&mut points);

    c.bench_function("voronoi from delaunay", |b| {
        b.iter(|| voronoi::delaunay_to_voronoi(&delaunay_dcel))
    });
}

// ==============================
// MapGenerator pipeline benchmarks
// ==============================

fn bench_map_initialize(c: &mut Criterion) {
    c.bench_function("map_generator initialize (small)", |b| {
        b.iter(|| {
            let extents = Extents2d::new(-5.0, -3.0, 5.0, 3.0);
            let rng = GlibcRand::new(42);
            let mut gen = MapGenerator::new(extents, 0.3, 640, 360, rng);
            gen.initialize();
        })
    });
}

fn bench_map_full_pipeline_small(c: &mut Criterion) {
    c.bench_function("map full pipeline (small 640x360)", |b| {
        b.iter(|| {
            let extents = Extents2d::new(-5.0, -3.0, 5.0, 3.0);
            let rng = GlibcRand::new(42);
            let mut gen = MapGenerator::new(extents, 0.3, 640, 360, rng);
            gen.initialize();
            gen.add_hill(0.0, 0.0, 3.0, 1.0);
            gen.add_hill(-2.0, 1.0, 2.0, 0.7);
            gen.normalize();
            gen.set_sea_level_to_median();
            gen.erode(0.5);
            gen.add_city("Capital".to_string(), "KINGDOM".to_string());
            gen.add_town("Village".to_string());
            gen.get_draw_data()
        })
    });
}

fn bench_terrain_erode(c: &mut Criterion) {
    // Pre-build a generator with terrain ready for erosion
    let extents = Extents2d::new(-5.0, -3.0, 5.0, 3.0);
    let rng = GlibcRand::new(42);
    let mut template = MapGenerator::new(extents, 0.3, 640, 360, rng);
    template.initialize();
    template.add_hill(0.0, 0.0, 3.0, 1.0);
    template.normalize();
    template.set_sea_level_to_median();

    c.bench_function("erode (small map)", |b| {
        b.iter_batched(
            || {
                // Clone the pre-built state before each iteration
                let extents = Extents2d::new(-5.0, -3.0, 5.0, 3.0);
                let rng = GlibcRand::new(42);
                let mut gen = MapGenerator::new(extents, 0.3, 640, 360, rng);
                gen.initialize();
                gen.add_hill(0.0, 0.0, 3.0, 1.0);
                gen.normalize();
                gen.set_sea_level_to_median();
                gen
            },
            |mut gen| gen.erode(0.5),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_export_heightmap(c: &mut Criterion) {
    let extents = Extents2d::new(-5.0, -3.0, 5.0, 3.0);
    let rng = GlibcRand::new(42);
    let mut gen = MapGenerator::new(extents, 0.3, 640, 360, rng);
    gen.initialize();
    gen.add_hill(0.0, 0.0, 3.0, 1.0);
    gen.normalize();
    gen.set_sea_level_to_median();
    gen.erode(0.5);

    c.bench_function("export_heightmap 640x360", |b| {
        b.iter(|| gen.export_heightmap(640, 360))
    });
}

// ==============================
// Criterion groups
// ==============================

criterion_group!(
    algorithm_benches,
    bench_poisson_disc_sampling,
    bench_delaunay_triangulation,
    bench_voronoi_construction,
);

criterion_group!(
    map_generator_benches,
    bench_map_initialize,
    bench_map_full_pipeline_small,
    bench_terrain_erode,
    bench_export_heightmap,
);

criterion_main!(algorithm_benches, map_generator_benches);
