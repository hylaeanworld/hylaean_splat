# Test Datasets Documentation

This document describes the test datasets used for validating COLMAP and Brush integrations in Hylaean Splat.

## Dataset Categories

### Small Dataset - Quick Validation
**Purpose**: Fast development testing and continuous integration

**Specifications**:
- **Scene**: Simple textured cube on plain background
- **Images**: 12 photos in circular pattern
- **Resolution**: 512x512 pixels
- **Format**: PNG (lossless)
- **Lighting**: Uniform LED lighting
- **Camera**: Fixed focal length, known intrinsics
- **Expected Processing Time**: 2-5 minutes total

**Capture Details**:
- **Object**: 10cm textured cube with distinct patterns
- **Background**: White seamless backdrop
- **Camera Path**: 30° elevation, 360° rotation, 12 positions
- **Focal Length**: 50mm equivalent
- **Aperture**: f/8 for sharp focus
- **ISO**: 100 for minimal noise

**Expected Results**:
- **COLMAP**: ~500-1000 sparse points
- **Dense Points**: ~50,000 points
- **Reconstruction Error**: <0.5 pixels
- **Completeness**: >95% surface coverage

### Medium Dataset - Realistic Testing
**Purpose**: Real-world performance validation

**Specifications**:
- **Scene**: Indoor room with furniture and decorations
- **Images**: 80 photos with varied viewpoints
- **Resolution**: 1920x1080 pixels
- **Format**: JPG (high quality)
- **Lighting**: Mixed natural and artificial
- **Camera**: Smartphone with EXIF data
- **Expected Processing Time**: 15-30 minutes total

**Capture Details**:
- **Environment**: 4m x 3m furnished room
- **Features**: Textured walls, wooden furniture, books, plants
- **Camera Path**: Handheld capture with overlapping views
- **Focal Length**: Variable (28-35mm equivalent)
- **Exposure**: Automatic with exposure compensation
- **Coverage**: Floor, walls, ceiling, and objects

**Expected Results**:
- **COLMAP**: ~5,000-10,000 sparse points
- **Dense Points**: ~500,000 points
- **Reconstruction Error**: <1.0 pixels
- **Completeness**: >90% surface coverage

### Challenging Dataset - Stress Testing
**Purpose**: Edge case handling and robustness validation

**Specifications**:
- **Scene**: Outdoor environment with challenging conditions
- **Images**: 150 photos with difficult scenarios
- **Resolution**: 4000x3000 pixels
- **Format**: RAW + JPG pairs
- **Lighting**: Variable outdoor conditions
- **Camera**: DSLR with manual settings
- **Expected Processing Time**: 1-2 hours total

**Capture Details**:
- **Environment**: Garden with trees, reflective surfaces, shadows
- **Challenges**: Motion blur, changing lighting, reflections
- **Camera Path**: Complex trajectory with close-ups and wide shots
- **Focal Length**: 24-70mm zoom lens
- **Conditions**: Partly cloudy day with moving shadows
- **Difficulties**: Specular surfaces, fine details, occlusions

**Expected Results**:
- **COLMAP**: ~20,000-50,000 sparse points
- **Dense Points**: ~2,000,000 points
- **Reconstruction Error**: <2.0 pixels
- **Completeness**: >80% surface coverage

## Dataset Structure

### Directory Organization
```
testing/datasets/
├── small/
│   ├── images/              # Source images
│   │   ├── IMG_0001.png
│   │   ├── IMG_0002.png
│   │   └── ...
│   ├── ground_truth/        # Reference data
│   │   ├── cameras.txt      # Known camera parameters
│   │   ├── points3D.ply     # Reference 3D points
│   │   └── poses.txt        # Ground truth poses
│   ├── metadata/            # Additional information
│   │   ├── capture_info.json
│   │   └── scene_description.txt
│   └── README.md           # Dataset-specific documentation
```

### Metadata Format
```json
{
  "dataset_name": "small_cube",
  "version": "1.0",
  "creation_date": "2025-01-01",
  "scene_description": "Textured cube on white background",
  "capture_device": "Canon EOS R5",
  "lighting_conditions": "Controlled LED lighting",
  "camera_parameters": {
    "focal_length_mm": 50,
    "sensor_width_mm": 36,
    "image_width_px": 512,
    "image_height_px": 512
  },
  "quality_metrics": {
    "blur_score": 0.95,
    "noise_level": 0.02,
    "dynamic_range": 8.5
  }
}
```

## Dataset Generation

### Synthetic Data Generation
For controlled testing, we can generate synthetic datasets:

```python
# Synthetic dataset generation
import bpy  # Blender Python API
import numpy as np

def generate_synthetic_dataset(output_dir, num_views=12):
    """Generate synthetic cube dataset with known ground truth"""
    
    # Setup scene
    setup_cube_scene()
    setup_lighting()
    
    # Generate camera positions
    camera_positions = generate_camera_circle(radius=1.0, num_views=num_views)
    
    # Render views
    for i, (pos, rot) in enumerate(camera_positions):
        set_camera_transform(pos, rot)
        render_image(f"{output_dir}/IMG_{i:04d}.png")
        save_camera_parameters(f"{output_dir}/cameras/{i:04d}.json")
    
    # Export ground truth
    export_ground_truth_mesh(f"{output_dir}/ground_truth/mesh.ply")
    export_camera_trajectory(f"{output_dir}/ground_truth/trajectory.json")
```

### Real Data Capture Guidelines

#### Equipment Recommendations
- **Camera**: DSLR or mirrorless with manual controls
- **Lens**: Fixed focal length preferred (35-50mm)
- **Tripod**: For consistent height and stability
- **Lighting**: External flash or LED panels for indoor scenes

#### Capture Patterns
- **Circular Pattern**: For simple objects
- **Grid Pattern**: For planar scenes
- **Spiral Pattern**: For complex 3D structures
- **Random Walk**: For natural exploration

#### Quality Checklist
- [ ] Sufficient image overlap (>60%)
- [ ] Consistent exposure across sequence
- [ ] Sharp focus throughout
- [ ] Minimal motion blur
- [ ] Diverse viewing angles
- [ ] Adequate lighting

## Dataset Validation

### Automatic Quality Checks
```python
def validate_dataset(dataset_path):
    """Validate dataset quality before processing"""
    
    issues = []
    
    # Check image count
    if len(images) < 8:
        issues.append("Insufficient images for reconstruction")
    
    # Check image quality
    for img in images:
        if calculate_blur_score(img) < 0.7:
            issues.append(f"Blurry image detected: {img.name}")
        
        if calculate_exposure_quality(img) < 0.6:
            issues.append(f"Poor exposure: {img.name}")
    
    # Check overlap
    overlap_matrix = calculate_image_overlap(images)
    if np.mean(overlap_matrix) < 0.4:
        issues.append("Insufficient image overlap")
    
    return issues
```

### Ground Truth Validation
```python
def validate_ground_truth(gt_path, reconstruction_path):
    """Compare reconstruction against ground truth"""
    
    gt_points = load_point_cloud(f"{gt_path}/points3D.ply")
    recon_points = load_point_cloud(f"{reconstruction_path}/points3D.ply")
    
    # Calculate metrics
    metrics = {
        "completeness": calculate_completeness(gt_points, recon_points),
        "accuracy": calculate_accuracy(gt_points, recon_points),
        "chamfer_distance": calculate_chamfer_distance(gt_points, recon_points),
        "f_score": calculate_f_score(gt_points, recon_points)
    }
    
    return metrics
```

## Performance Benchmarks

### Expected Performance Targets

#### Small Dataset
- **COLMAP Feature Extraction**: <30 seconds
- **COLMAP Matching**: <15 seconds
- **COLMAP Mapping**: <30 seconds
- **Brush Training**: <2 minutes
- **Brush Rendering**: <5 seconds per view

#### Medium Dataset
- **COLMAP Feature Extraction**: <5 minutes
- **COLMAP Matching**: <3 minutes
- **COLMAP Mapping**: <10 minutes
- **Brush Training**: <15 minutes
- **Brush Rendering**: <10 seconds per view

#### Challenging Dataset
- **COLMAP Feature Extraction**: <30 minutes
- **COLMAP Matching**: <20 minutes
- **COLMAP Mapping**: <45 minutes
- **Brush Training**: <60 minutes
- **Brush Rendering**: <30 seconds per view

### Quality Thresholds

#### COLMAP Reconstruction
- **Reprojection Error**: <1.0 pixels
- **Track Length**: >3 observations per point
- **Registered Images**: >90% of input images

#### Brush Training
- **PSNR**: >20 dB on test views
- **SSIM**: >0.8 on test views
- **Training Loss**: Converged (stable for 100 iterations)

#### Rendering Quality
- **Frame Rate**: >30 FPS for interactive viewing
- **Memory Usage**: <4GB GPU memory
- **Rendering Artifacts**: Minimal ghosting or holes

## Dataset Download and Setup

### Automatic Download Script
```bash
#!/bin/bash
# Download and setup test datasets

DATASETS_URL="https://datasets.hylaean.com"
TARGET_DIR="testing/datasets"

# Create directories
mkdir -p "$TARGET_DIR"/{small,medium,challenging}

# Download datasets
echo "Downloading small dataset..."
wget -q --show-progress "$DATASETS_URL/small.tar.gz" -O small.tar.gz
tar -xzf small.tar.gz -C "$TARGET_DIR/small/"

echo "Downloading medium dataset..."
wget -q --show-progress "$DATASETS_URL/medium.tar.gz" -O medium.tar.gz
tar -xzf medium.tar.gz -C "$TARGET_DIR/medium/"

echo "Downloading challenging dataset..."
wget -q --show-progress "$DATASETS_URL/challenging.tar.gz" -O challenging.tar.gz
tar -xzf challenging.tar.gz -C "$TARGET_DIR/challenging/"

# Cleanup
rm *.tar.gz

echo "Dataset setup complete!"
```

### Manual Setup Instructions
If automatic download fails:

1. **Download Archives**: Get dataset files from backup locations
2. **Extract**: Unzip to appropriate directories
3. **Verify**: Check file integrity with provided checksums
4. **Validate**: Run dataset validation scripts

## Contributing New Datasets

### Dataset Requirements
- **Minimum 8 images** for basic reconstruction
- **Ground truth data** for validation
- **Metadata file** with capture information
- **Documentation** explaining scene and challenges

### Submission Process
1. Create dataset following structure guidelines
2. Generate quality metrics and benchmarks
3. Test with existing pipeline
4. Submit via pull request with documentation

### Review Criteria
- **Technical Quality**: Sharp, well-exposed images
- **Diversity**: Various challenging conditions
- **Documentation**: Clear descriptions and metadata
- **Validation**: Proven reconstruction quality
- **Licensing**: Clear usage permissions

This comprehensive dataset framework ensures robust testing across various scenarios while maintaining reproducible results.