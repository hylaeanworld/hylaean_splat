# Hylaean Splat Testing Framework

This directory contains comprehensive testing resources for COLMAP and Brush integrations.

## Directory Structure

```
testing/
├── README.md                 # This file
├── datasets/                 # Sample test datasets
│   ├── small/               # 10-20 images for quick testing
│   ├── medium/              # 50-100 images for realistic testing
│   └── challenging/         # Complex scenarios
├── scripts/                 # Installation and test scripts
│   ├── install_colmap.sh    # COLMAP installation script
│   ├── install_brush.sh     # Brush installation script
│   ├── setup_test_env.sh    # Complete environment setup
│   └── run_full_pipeline.sh # End-to-end pipeline test
├── outputs/                 # Test output directory
│   ├── colmap_results/      # COLMAP reconstruction results
│   ├── brush_results/       # Brush training results
│   └── renders/             # Final rendered outputs
└── validation/              # Validation scripts and expected results
    ├── validate_colmap.py   # COLMAP output validation
    ├── validate_brush.py    # Brush output validation
    └── expected_results/    # Reference outputs for comparison
```

## Quick Start

1. **Setup Environment**: `./scripts/setup_test_env.sh`
2. **Download Sample Data**: `./scripts/download_test_data.sh`
3. **Run Full Pipeline**: `./scripts/run_full_pipeline.sh small`

## Test Datasets

### Small Dataset (Quick Testing)
- **Images**: 12 photos of a simple object
- **Capture**: Single object, controlled lighting
- **Expected Runtime**: 5-10 minutes for full pipeline
- **Use Case**: Development and quick validation

### Medium Dataset (Realistic Testing)
- **Images**: 80 photos of an indoor scene
- **Capture**: Room with furniture, natural lighting
- **Expected Runtime**: 30-60 minutes for full pipeline
- **Use Case**: Performance testing and quality validation

### Challenging Dataset (Stress Testing)
- **Images**: 150 photos with complex scenarios
- **Capture**: Mixed lighting, reflective surfaces, motion blur
- **Expected Runtime**: 2-3 hours for full pipeline
- **Use Case**: Robustness testing and edge case handling

## Installation Scripts

### COLMAP Installation
```bash
./scripts/install_colmap.sh
```

Supports multiple installation methods:
- **Ubuntu/Debian**: `apt` package manager
- **macOS**: `brew` package manager
- **Conda**: `conda` package manager
- **Source**: Build from source with CUDA support

### Brush Installation
```bash
./scripts/install_brush.sh
```

Handles:
- Rust toolchain setup
- GPU dependencies (CUDA/ROCm)
- Build optimization flags
- Installation verification

## Pipeline Testing

### Full Pipeline Test
```bash
./scripts/run_full_pipeline.sh [dataset_size]
```

Where `dataset_size` is one of: `small`, `medium`, `challenging`

This script:
1. Initializes Hylaean Splat
2. Installs required tools locally
3. Runs COLMAP full pipeline
4. Converts COLMAP output for Brush
5. Trains Gaussian Splat model with Brush
6. Renders test views
7. Validates outputs

### Individual Tool Testing
```bash
# Test COLMAP only
./scripts/test_colmap.sh [dataset_size]

# Test Brush only (requires COLMAP output)
./scripts/test_brush.sh [dataset_size]
```

## Validation

### Automated Validation
```bash
./validation/validate_pipeline.py [output_directory]
```

Checks:
- COLMAP reconstruction quality
- Point cloud density and coverage
- Brush training convergence
- Rendering quality metrics
- Performance benchmarks

### Manual Validation
- Visual inspection of reconstruction
- Comparison with expected results
- Performance profiling
- Error log analysis

## Configuration

### Environment Variables
```bash
# Optional: Specify tool installation directory
export HYLAEAN_TOOLS_DIR="./tools"

# Optional: Enable verbose logging
export HYLAEAN_LOG_LEVEL="debug"

# Optional: CUDA device selection
export CUDA_VISIBLE_DEVICES="0"
```

### Test Configuration
Edit `testing/config.toml` to customize:
- Dataset download URLs
- Quality thresholds
- Performance targets
- Output formats

## Troubleshooting

### Common Issues

1. **COLMAP Installation Fails**
   - Check CUDA compatibility
   - Verify system dependencies
   - Try conda installation as fallback

2. **Brush Build Fails**
   - Ensure Rust toolchain is up to date
   - Check GPU driver compatibility
   - Verify system has sufficient RAM

3. **Pipeline Hangs**
   - Check available disk space
   - Monitor GPU memory usage
   - Verify image format compatibility

### Debug Mode
```bash
HYLAEAN_LOG_LEVEL=debug ./scripts/run_full_pipeline.sh small
```

### Performance Monitoring
```bash
# Monitor GPU usage
nvidia-smi -l 1

# Monitor system resources
htop

# Check pipeline logs
tail -f ~/.hylaean_splat/logs/pipeline.log
```

## Expected Results

### Small Dataset
- **COLMAP**: ~1000 sparse points, 12 camera poses
- **Brush**: Training converges in ~1000 iterations
- **Rendering**: 512x512 novel views in <1 second

### Medium Dataset
- **COLMAP**: ~10,000 sparse points, 80 camera poses
- **Brush**: Training converges in ~5000 iterations
- **Rendering**: 1024x1024 novel views in <2 seconds

### Challenging Dataset
- **COLMAP**: ~50,000 sparse points, 150 camera poses
- **Brush**: Training converges in ~10,000 iterations
- **Rendering**: 2048x2048 novel views in <5 seconds

## Contributing

To add new test datasets:
1. Create directory in `datasets/`
2. Add download script to `scripts/`
3. Update validation scripts
4. Document expected results