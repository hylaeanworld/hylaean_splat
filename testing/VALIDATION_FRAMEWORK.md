# Validation Framework Documentation

This document describes the comprehensive validation framework for testing COLMAP and Brush integrations in Hylaean Splat.

## Overview

The validation framework provides automated quality assessment, performance benchmarking, and regression testing for the complete 3D reconstruction pipeline.

## Validation Components

### 1. Input Data Validation
**Purpose**: Ensure test datasets meet quality requirements before processing

**Components**:
- **Image Quality Assessment**: Blur detection, exposure analysis, noise evaluation
- **Dataset Completeness**: Verify all required files are present
- **Metadata Validation**: Check capture parameters and ground truth data
- **Overlap Analysis**: Ensure sufficient image overlap for reconstruction

### 2. COLMAP Output Validation
**Purpose**: Validate structure-from-motion reconstruction quality

**Components**:
- **Sparse Reconstruction**: Point cloud density, reprojection errors
- **Camera Registration**: Pose accuracy, calibration quality
- **Feature Matching**: Match quality, track lengths
- **Geometric Consistency**: 3D point triangulation accuracy

### 3. Brush Training Validation
**Purpose**: Ensure Gaussian Splatting training converges properly

**Components**:
- **Training Convergence**: Loss curves, gradient norms
- **Model Quality**: PSNR, SSIM on validation views
- **Rendering Performance**: Frame rates, memory usage
- **Visual Quality**: Artifact detection, completeness

### 4. End-to-End Pipeline Validation
**Purpose**: Comprehensive system testing with quality metrics

**Components**:
- **Pipeline Integrity**: All stages complete successfully
- **Performance Benchmarks**: Timing, resource usage
- **Quality Metrics**: Reconstruction accuracy, rendering quality
- **Regression Testing**: Compare against previous results

## Validation Scripts

### Image Quality Validation (`validate_images.py`)

```python
import cv2
import numpy as np
from pathlib import Path
import json

class ImageQualityValidator:
    def __init__(self, min_blur_score=0.7, min_exposure_score=0.6):
        self.min_blur_score = min_blur_score
        self.min_exposure_score = min_exposure_score
        self.results = []
    
    def validate_dataset(self, dataset_path):
        """Validate all images in dataset"""
        image_dir = Path(dataset_path) / "images"
        images = list(image_dir.glob("*.png")) + list(image_dir.glob("*.jpg"))
        
        for img_path in images:
            result = self.validate_image(img_path)
            self.results.append(result)
        
        return self.generate_report()
    
    def validate_image(self, img_path):
        """Validate single image quality"""
        img = cv2.imread(str(img_path))
        
        # Blur detection using Laplacian variance
        gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
        blur_score = cv2.Laplacian(gray, cv2.CV_64F).var()
        blur_score = min(blur_score / 1000.0, 1.0)  # Normalize
        
        # Exposure analysis
        exposure_score = self.calculate_exposure_quality(img)
        
        # Noise estimation
        noise_level = self.estimate_noise_level(gray)
        
        return {
            "image": img_path.name,
            "blur_score": blur_score,
            "exposure_score": exposure_score,
            "noise_level": noise_level,
            "resolution": img.shape[:2],
            "passed": blur_score >= self.min_blur_score and 
                     exposure_score >= self.min_exposure_score
        }
    
    def calculate_exposure_quality(self, img):
        """Calculate exposure quality score"""
        # Convert to HSV for better exposure analysis
        hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
        v_channel = hsv[:, :, 2]
        
        # Calculate histogram
        hist = cv2.calcHist([v_channel], [0], None, [256], [0, 256])
        hist = hist.flatten() / hist.sum()
        
        # Penalize over/under exposure
        underexposed = hist[:25].sum()
        overexposed = hist[230:].sum()
        
        # Good exposure has most pixels in mid-range
        well_exposed = hist[50:200].sum()
        
        return well_exposed - (underexposed + overexposed) * 2
    
    def estimate_noise_level(self, gray_img):
        """Estimate noise level in image"""
        # Use median filter approach
        kernel = np.ones((3, 3), np.float32) / 9
        filtered = cv2.filter2D(gray_img, -1, kernel)
        noise = np.abs(gray_img.astype(np.float32) - filtered.astype(np.float32))
        return np.mean(noise)
```

### COLMAP Validation (`validate_colmap.py`)

```python
import numpy as np
from pathlib import Path
import sqlite3
import struct

class ColmapValidator:
    def __init__(self, max_reprojection_error=1.0, min_track_length=3):
        self.max_reprojection_error = max_reprojection_error
        self.min_track_length = min_track_length
    
    def validate_reconstruction(self, colmap_path):
        """Validate COLMAP reconstruction quality"""
        colmap_path = Path(colmap_path)
        
        # Validate sparse reconstruction
        sparse_results = self.validate_sparse_reconstruction(colmap_path)
        
        # Validate camera registration
        camera_results = self.validate_camera_registration(colmap_path)
        
        # Validate feature matching
        matching_results = self.validate_feature_matching(colmap_path)
        
        return {
            "sparse_reconstruction": sparse_results,
            "camera_registration": camera_results,
            "feature_matching": matching_results,
            "overall_quality": self.calculate_overall_quality(
                sparse_results, camera_results, matching_results
            )
        }
    
    def validate_sparse_reconstruction(self, colmap_path):
        """Validate sparse point cloud quality"""
        # Read points3D.bin or points3D.txt
        points_file = colmap_path / "sparse" / "0" / "points3D.bin"
        if not points_file.exists():
            points_file = colmap_path / "sparse" / "0" / "points3D.txt"
        
        if not points_file.exists():
            return {"error": "No sparse reconstruction found"}
        
        # Parse points and calculate statistics
        points = self.read_points3d_file(points_file)
        
        # Calculate metrics
        num_points = len(points)
        mean_error = np.mean([p['error'] for p in points])
        mean_track_length = np.mean([len(p['track']) for p in points])
        
        return {
            "num_points": num_points,
            "mean_reprojection_error": mean_error,
            "mean_track_length": mean_track_length,
            "quality_score": self.calculate_sparse_quality_score(
                num_points, mean_error, mean_track_length
            )
        }
    
    def validate_camera_registration(self, colmap_path):
        """Validate camera pose estimation"""
        images_file = colmap_path / "sparse" / "0" / "images.bin"
        if not images_file.exists():
            images_file = colmap_path / "sparse" / "0" / "images.txt"
        
        if not images_file.exists():
            return {"error": "No camera registration found"}
        
        # Read camera poses
        images = self.read_images_file(images_file)
        
        # Calculate registration statistics
        num_registered = len(images)
        registration_rate = num_registered / len(list(colmap_path.glob("images/*.jpg")))
        
        return {
            "num_registered": num_registered,
            "registration_rate": registration_rate,
            "quality_score": registration_rate
        }
    
    def read_points3d_file(self, file_path):
        """Read COLMAP points3D file"""
        if file_path.suffix == '.bin':
            return self.read_points3d_binary(file_path)
        else:
            return self.read_points3d_text(file_path)
    
    def calculate_sparse_quality_score(self, num_points, mean_error, mean_track_length):
        """Calculate quality score for sparse reconstruction"""
        # Normalize metrics
        point_density_score = min(num_points / 1000.0, 1.0)
        error_score = max(0, 1.0 - mean_error / self.max_reprojection_error)
        track_score = min(mean_track_length / self.min_track_length, 1.0)
        
        return (point_density_score + error_score + track_score) / 3.0
```

### Brush Training Validation (`validate_brush.py`)

```python
import json
import numpy as np
from pathlib import Path
import matplotlib.pyplot as plt

class BrushValidator:
    def __init__(self, min_psnr=20.0, min_ssim=0.8):
        self.min_psnr = min_psnr
        self.min_ssim = min_ssim
    
    def validate_training(self, model_path):
        """Validate Brush training results"""
        model_path = Path(model_path)
        
        # Validate training convergence
        convergence_results = self.validate_convergence(model_path)
        
        # Validate rendering quality
        quality_results = self.validate_rendering_quality(model_path)
        
        # Validate performance
        performance_results = self.validate_performance(model_path)
        
        return {
            "convergence": convergence_results,
            "quality": quality_results,
            "performance": performance_results,
            "overall_score": self.calculate_overall_score(
                convergence_results, quality_results, performance_results
            )
        }
    
    def validate_convergence(self, model_path):
        """Check if training converged properly"""
        # Read training log
        log_file = model_path / "training_log.json"
        if not log_file.exists():
            return {"error": "No training log found"}
        
        with open(log_file, 'r') as f:
            training_log = json.load(f)
        
        # Extract loss curves
        iterations = training_log.get("iterations", [])
        losses = training_log.get("losses", [])
        
        if not losses:
            return {"error": "No loss data found"}
        
        # Check convergence
        recent_losses = losses[-100:]  # Last 100 iterations
        loss_std = np.std(recent_losses)
        final_loss = losses[-1]
        
        converged = loss_std < 0.01 and final_loss < 0.1
        
        return {
            "converged": converged,
            "final_loss": final_loss,
            "loss_stability": loss_std,
            "total_iterations": len(iterations)
        }
    
    def validate_rendering_quality(self, model_path):
        """Validate rendering quality metrics"""
        # Read quality metrics
        metrics_file = model_path / "quality_metrics.json"
        if not metrics_file.exists():
            return {"error": "No quality metrics found"}
        
        with open(metrics_file, 'r') as f:
            metrics = json.load(f)
        
        psnr = metrics.get("psnr", 0)
        ssim = metrics.get("ssim", 0)
        lpips = metrics.get("lpips", 1.0)
        
        return {
            "psnr": psnr,
            "ssim": ssim,
            "lpips": lpips,
            "quality_score": self.calculate_quality_score(psnr, ssim, lpips)
        }
    
    def calculate_quality_score(self, psnr, ssim, lpips):
        """Calculate overall quality score"""
        psnr_score = min(psnr / 30.0, 1.0)  # Normalize to 30 dB
        ssim_score = ssim
        lpips_score = max(0, 1.0 - lpips / 0.5)  # Lower is better
        
        return (psnr_score + ssim_score + lpips_score) / 3.0
```

### Performance Benchmarking (`benchmark_performance.py`)

```python
import time
import psutil
import subprocess
import json
from pathlib import Path

class PerformanceBenchmark:
    def __init__(self):
        self.results = {}
    
    def benchmark_full_pipeline(self, dataset_path, output_path):
        """Benchmark complete pipeline performance"""
        dataset_path = Path(dataset_path)
        output_path = Path(output_path)
        
        # Benchmark each stage
        self.benchmark_colmap_pipeline(dataset_path, output_path)
        self.benchmark_brush_training(output_path)
        self.benchmark_brush_rendering(output_path)
        
        return self.results
    
    def benchmark_colmap_pipeline(self, dataset_path, output_path):
        """Benchmark COLMAP performance"""
        stages = [
            ("feature_extraction", self.run_colmap_feature_extraction),
            ("feature_matching", self.run_colmap_matching),
            ("sparse_reconstruction", self.run_colmap_mapping)
        ]
        
        self.results["colmap"] = {}
        
        for stage_name, stage_func in stages:
            start_time = time.time()
            start_memory = psutil.virtual_memory().used
            
            # Run stage
            stage_func(dataset_path, output_path)
            
            end_time = time.time()
            end_memory = psutil.virtual_memory().used
            
            self.results["colmap"][stage_name] = {
                "duration": end_time - start_time,
                "memory_usage": end_memory - start_memory,
                "timestamp": time.time()
            }
    
    def benchmark_brush_training(self, model_path):
        """Benchmark Brush training performance"""
        start_time = time.time()
        start_memory = psutil.virtual_memory().used
        
        # Monitor GPU usage if available
        gpu_stats = self.monitor_gpu_usage()
        
        # Training would be running here
        # For now, we'll simulate or read from actual training
        
        end_time = time.time()
        end_memory = psutil.virtual_memory().used
        
        self.results["brush_training"] = {
            "duration": end_time - start_time,
            "memory_usage": end_memory - start_memory,
            "gpu_stats": gpu_stats,
            "timestamp": time.time()
        }
    
    def monitor_gpu_usage(self):
        """Monitor GPU usage during processing"""
        try:
            result = subprocess.run(
                ["nvidia-smi", "--query-gpu=utilization.gpu,memory.used,memory.total", 
                 "--format=csv,noheader,nounits"],
                capture_output=True, text=True
            )
            
            if result.returncode == 0:
                lines = result.stdout.strip().split('\n')
                gpu_data = []
                for line in lines:
                    util, mem_used, mem_total = line.split(', ')
                    gpu_data.append({
                        "utilization": int(util),
                        "memory_used": int(mem_used),
                        "memory_total": int(mem_total)
                    })
                return gpu_data
        except:
            pass
        
        return None
```

## Quality Metrics

### Reconstruction Quality Metrics

#### Geometric Accuracy
- **Chamfer Distance**: Average distance between reconstructed and ground truth points
- **Hausdorff Distance**: Maximum distance between point sets
- **Completeness**: Percentage of ground truth points within threshold
- **Accuracy**: Percentage of reconstructed points within threshold

#### Visual Quality Metrics
- **PSNR**: Peak Signal-to-Noise Ratio for rendered images
- **SSIM**: Structural Similarity Index for perceptual quality
- **LPIPS**: Learned Perceptual Image Patch Similarity

#### Performance Metrics
- **Processing Time**: Total time for each pipeline stage
- **Memory Usage**: Peak memory consumption
- **GPU Utilization**: GPU usage during processing
- **Throughput**: Images processed per second

### Validation Thresholds

#### Small Dataset Targets
- **COLMAP Reprojection Error**: <0.5 pixels
- **Brush PSNR**: >25 dB
- **Brush SSIM**: >0.85
- **Processing Time**: <5 minutes total

#### Medium Dataset Targets
- **COLMAP Reprojection Error**: <1.0 pixels
- **Brush PSNR**: >22 dB
- **Brush SSIM**: >0.80
- **Processing Time**: <30 minutes total

#### Challenging Dataset Targets
- **COLMAP Reprojection Error**: <2.0 pixels
- **Brush PSNR**: >20 dB
- **Brush SSIM**: >0.75
- **Processing Time**: <2 hours total

## Automated Testing

### Continuous Integration Pipeline

```yaml
name: Pipeline Validation
on: [push, pull_request]

jobs:
  test-small-dataset:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup environment
        run: ./testing/scripts/setup_test_env.sh
      - name: Run pipeline test
        run: ./testing/scripts/run_full_pipeline.sh small
      - name: Validate results
        run: python3 testing/validation/validate_pipeline.py testing/outputs/small
  
  test-medium-dataset:
    runs-on: ubuntu-latest
    needs: test-small-dataset
    steps:
      - uses: actions/checkout@v2
      - name: Setup environment
        run: ./testing/scripts/setup_test_env.sh
      - name: Run pipeline test
        run: ./testing/scripts/run_full_pipeline.sh medium
      - name: Validate results
        run: python3 testing/validation/validate_pipeline.py testing/outputs/medium
```

### Regression Testing

```python
def run_regression_tests():
    """Compare current results with baseline"""
    current_results = load_current_results()
    baseline_results = load_baseline_results()
    
    regressions = []
    
    for metric in current_results:
        if metric in baseline_results:
            current_value = current_results[metric]
            baseline_value = baseline_results[metric]
            
            # Check for significant degradation
            if current_value < baseline_value * 0.95:  # 5% threshold
                regressions.append({
                    "metric": metric,
                    "current": current_value,
                    "baseline": baseline_value,
                    "degradation": (baseline_value - current_value) / baseline_value
                })
    
    return regressions
```

## Reporting and Visualization

### Validation Report Generation

```python
def generate_validation_report(results):
    """Generate comprehensive validation report"""
    
    report = {
        "timestamp": time.time(),
        "dataset": results["dataset_name"],
        "pipeline_version": get_pipeline_version(),
        "summary": {
            "passed": results["validation_passed"],
            "quality_score": results["overall_quality_score"],
            "performance_score": results["performance_score"]
        },
        "details": results
    }
    
    # Generate HTML report
    html_report = generate_html_report(report)
    
    # Generate plots
    generate_quality_plots(results)
    generate_performance_plots(results)
    
    return report
```

### Dashboard Integration

The validation framework integrates with a web dashboard showing:
- **Real-time Pipeline Status**: Current test progress
- **Historical Performance**: Trends over time
- **Quality Metrics**: Visual quality comparisons
- **Regression Analysis**: Performance degradation detection

This comprehensive validation framework ensures consistent quality and performance across all pipeline components while providing detailed insights for optimization and debugging.