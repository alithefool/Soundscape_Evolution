use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Global configuration for Soundscape Evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub window: WindowConfig,
    pub audio: AudioConfig,
    pub simulation: SimulationConfig,
    pub visualization: VisualizationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub fft_size: usize,
    pub bass_range: (f32, f32),    // Hz range for bass frequencies
    pub mid_range: (f32, f32),     // Hz range for mid frequencies
    pub treble_range: (f32, f32),  // Hz range for treble frequencies
    pub sensitivity: f32,          // Overall audio sensitivity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub width: usize,
    pub height: usize,
    pub update_rate: f32,         // Updates per second
    pub initial_seed: f32,        // Random seed density (0.0-1.0)
    pub edge_behavior: EdgeBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeBehavior {
    Wrap,      // Cells wrap around edges
    Dead,      // Cells outside the grid are considered dead
    Alive,     // Cells outside the grid are considered alive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub cell_size: u32,           // Size of each cell in pixels
    pub color_scheme: ColorScheme,
    pub fade_rate: f32,           // Rate at which dead cells fade out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    Classic,    // Black and white
    Heat,       // Heat map (blue to red)
    Rainbow,    // Full color spectrum
    Pulse,      // Color changes with audio pulse
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
    
    pub fn default() -> Self {
        Config {
            window: WindowConfig {
                title: "Soundscape Evolution".to_string(),
                width: 800,
                height: 600,
                fullscreen: false,
            },
            audio: AudioConfig {
                sample_rate: 44100,
                channels: 2,
                fft_size: 2048,
                bass_range: (20.0, 250.0),
                mid_range: (250.0, 2000.0),
                treble_range: (2000.0, 20000.0),
                sensitivity: 1.0,
            },
            simulation: SimulationConfig {
                width: 200,
                height: 150,
                update_rate: 30.0,
                initial_seed: 0.3,
                edge_behavior: EdgeBehavior::Wrap,
            },
            visualization: VisualizationConfig {
                cell_size: 4,
                color_scheme: ColorScheme::Pulse,
                fade_rate: 0.1,
            },
        }
    }
}