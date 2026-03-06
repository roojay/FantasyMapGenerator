# Baseline Reference Data

This directory contains the baseline reference data generated from the C++ implementation of the Fantasy Map Generator. This data serves as the "ground truth" for validating the Rust migration.

## Generation Parameters

The baseline data was generated with the following parameters:

```bash
./map_generation -v -s 12345 -r 0.08 -o baseline/reference.json
```

- **Seed**: `12345` (fixed for reproducibility)
- **Resolution**: `0.08` (level of map detail)
- **Output**: JSON format (drawing support not available in build environment)
- **Verbose**: Enabled for detailed logging

## Generated Data

- `reference.json` - JSON drawing data (1.1MB) containing:
  - City positions and names
  - Contour lines (elevation)
  - River paths
  - Territory borders
  - Town positions and names
  - Label placements
  - Slope shading data

- `generation.log` - Detailed generation log showing timing for each step

## Algorithm Parameters (from log)

- **Poisson Disc Samples**: 72,180 points
- **Erosion**: 0.33184 amount over 3 iterations
- **Cities**: 5 (Niafunke/Markala, Menaka/Koulikoro, Bandiagara/Kinmparana, Kokofata/Gao, Kati/Kolokani)
- **Towns**: 16 (various Mali city names)
- **Total Generation Time**: 3.81184 seconds

## Validation Requirements

The Rust implementation must produce JSON output that matches this baseline:

1. **Structure**: All JSON keys must match
2. **Numeric Precision**: Floating-point values must match within epsilon=0.00001
3. **Array Lengths**: All arrays must have the same number of elements
4. **Determinism**: Running with the same seed (12345) must produce identical results

## Note on Rendering

The C++ build did not have PyCairo support, so no PNG image was generated. The Rust implementation will use WebGPU (wgpu) for rendering, but the JSON data serves as the primary validation point for algorithm correctness.
