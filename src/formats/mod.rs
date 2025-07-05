//! Data format handling and conversion utilities

pub mod point_cloud;
pub mod camera_params;

use crate::errors::{Result, HylaeanError};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    PointCloud(PointCloudFormat),
    CameraParameters(CameraFormat),
    Dataset(DatasetFormat),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PointCloudFormat {
    PLY,
    PCD,
    XYZ,
    LAZ,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraFormat {
    COLMAP,
    NeRF,
    OpenCV,
    Blender,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatasetFormat {
    NeRFSynthetic,
    LLFF,
    TanksAndTemples,
    Custom(String),
}

pub trait FormatConverter {
    fn can_convert(&self, from: &DataFormat, to: &DataFormat) -> bool;
    fn convert(&self, input_path: &Path, output_path: &Path, from: &DataFormat, to: &DataFormat) -> Result<()>;
}

pub fn detect_format(path: &Path) -> Result<DataFormat> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "ply" => Ok(DataFormat::PointCloud(PointCloudFormat::PLY)),
        "pcd" => Ok(DataFormat::PointCloud(PointCloudFormat::PCD)),
        "xyz" => Ok(DataFormat::PointCloud(PointCloudFormat::XYZ)),
        "laz" => Ok(DataFormat::PointCloud(PointCloudFormat::LAZ)),
        "txt" => {
            // Could be camera parameters or point cloud
            if looks_like_camera_params(path)? {
                Ok(DataFormat::CameraParameters(CameraFormat::COLMAP))
            } else {
                Ok(DataFormat::PointCloud(PointCloudFormat::XYZ))
            }
        }
        "json" => Ok(DataFormat::CameraParameters(CameraFormat::NeRF)),
        _ => Err(HylaeanError::UnsupportedFormat {
            format: extension.to_string(),
        }),
    }
}

fn looks_like_camera_params(path: &Path) -> Result<bool> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines().take(5) {
        let line = line?;
        if line.contains("# Camera list") || line.contains("# Image list") {
            return Ok(true);
        }
    }
    
    Ok(false)
}

pub fn parse_format(format_str: &str) -> Result<DataFormat> {
    match format_str.to_lowercase().as_str() {
        "ply" => Ok(DataFormat::PointCloud(PointCloudFormat::PLY)),
        "pcd" => Ok(DataFormat::PointCloud(PointCloudFormat::PCD)),
        "xyz" => Ok(DataFormat::PointCloud(PointCloudFormat::XYZ)),
        "laz" => Ok(DataFormat::PointCloud(PointCloudFormat::LAZ)),
        "colmap" => Ok(DataFormat::CameraParameters(CameraFormat::COLMAP)),
        "nerf" => Ok(DataFormat::CameraParameters(CameraFormat::NeRF)),
        "opencv" => Ok(DataFormat::CameraParameters(CameraFormat::OpenCV)),
        "blender" => Ok(DataFormat::CameraParameters(CameraFormat::Blender)),
        _ => Err(HylaeanError::UnsupportedFormat {
            format: format_str.to_string(),
        }),
    }
}