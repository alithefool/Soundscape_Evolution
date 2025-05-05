use crate::config::ColorScheme;
use crate::audio::analyzer::AudioFrame;

/// RGB color representation
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8, // Alpha channel
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
    
    pub fn black() -> Self {
        Color { r: 0, g: 0, b: 0, a: 255 }
    }
    
    pub fn white() -> Self {
        Color { r: 255, g: 255, b: 255, a: 255 }
    }
    
    pub fn to_rgba(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
    
    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.b as u32) << 16) | ((self.g as u32) << 8) | (self.r as u32)
    }
    
    /// Create a color with faded alpha based on the original color
    pub fn with_alpha(&self, alpha: u8) -> Self {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }
    
    /// Fade the color by a given factor (0.0 = transparent, 1.0 = opaque)
    pub fn fade(&self, factor: f32) -> Self {
        let alpha = (self.a as f32 * factor.clamp(0.0, 1.0)) as u8;
        self.with_alpha(alpha)
    }
}

/// ColorPalette handles different color schemes for the visualization
pub struct ColorPalette {
    scheme: ColorScheme,
    time: f32,                // Used for time-based effects
    audio_frame: Option<AudioFrame>, // Current audio frame for reactive effects
}

impl ColorPalette {
    pub fn new(scheme: ColorScheme) -> Self {
        ColorPalette {
            scheme,
            time: 0.0,
            audio_frame: None,
        }
    }
    
    /// Update the palette with new audio data and time
    pub fn update(&mut self, audio_frame: Option<&AudioFrame>, delta_time: f32) {
        if let Some(frame) = audio_frame {
            self.audio_frame = Some(frame.clone());
        }
        self.time += delta_time;
    }
    
    /// Set color scheme
    pub fn set_scheme(&mut self, scheme: ColorScheme) {
        self.scheme = scheme;
    }
    
    /// Get cell color based on its age and the current color scheme
    pub fn get_cell_color(&self, age: u8, max_age: u8) -> Color {
        match self.scheme {
            ColorScheme::Classic => {
                // Simple black and white
                if age > 0 {
                    Color::white()
                } else {
                    Color::black()
                }
            },
            ColorScheme::Heat => {
                // Heat map: Blue (cold) to Red (hot)
                if age == 0 {
                    return Color::black();
                }
                
                let normalized_age = age as f32 / max_age as f32;
                let r = (normalized_age * 255.0) as u8;
                let g = ((1.0 - normalized_age) * 255.0 * normalized_age) as u8;
                let b = ((1.0 - normalized_age) * 255.0) as u8;
                
                Color::new(r, g, b, 255)
            },
            ColorScheme::Rainbow => {
                // Rainbow colors based on cell age
                if age == 0 {
                    return Color::black();
                }
                
                // Hue based on age (0.0 to 1.0)
                let hue = (age as f32 / max_age as f32) * 6.0;
                let sector = hue.floor();
                let offset = hue - sector;
                
                let v = 255; // Value (brightness)
                let p = 0;   // Min value
                
                match sector as u8 % 6 {
                    0 => Color::new(v, (v as f32 * offset) as u8, p, 255),  // Red to Yellow
                    1 => Color::new((v as f32 * (1.0 - offset)) as u8, v, p, 255),  // Yellow to Green
                    2 => Color::new(p, v, (v as f32 * offset) as u8, 255),  // Green to Cyan
                    3 => Color::new(p, (v as f32 * (1.0 - offset)) as u8, v, 255),  // Cyan to Blue
                    4 => Color::new((v as f32 * offset) as u8, p, v, 255),  // Blue to Magenta
                    _ => Color::new(v, p, (v as f32 * (1.0 - offset)) as u8, 255),  // Magenta to Red
                }
            },
            ColorScheme::Pulse => {
                // Colors pulse with the audio
                if age == 0 {
                    return Color::black();
                }
                
                if let Some(ref frame) = self.audio_frame {
                    // Use audio energy to influence colors
                    let bass = frame.bass_energy.clamp(0.0, 1.0);
                    let mid = frame.mid_energy.clamp(0.0, 1.0);
                    let treble = frame.treble_energy.clamp(0.0, 1.0);
                    
                    // Age affects color intensity
                    let intensity = (age as f32 / max_age as f32).min(1.0);
                    
                    // Create pulsing colors based on audio bands
                    let r = (bass * 255.0 * intensity) as u8;
                    let g = (mid * 255.0 * intensity) as u8;
                    let b = (treble * 255.0 * intensity) as u8;
                    
                    Color::new(r, g, b, 255)
                } else {
                    // Default to white if no audio data
                    Color::white()
                }
            },
        }
    }
    
    /// Get background color based on audio energy
    pub fn get_background_color(&self) -> Color {
        match self.scheme {
            ColorScheme::Classic => Color::black(),
            ColorScheme::Heat => Color::new(0, 0, 20, 255), // Dark blue
            ColorScheme::Rainbow => Color::black(),
            ColorScheme::Pulse => {
                if let Some(ref frame) = self.audio_frame {
                    // Subtle background pulse with the audio
                    let energy = frame.overall_energy * 0.2; // Subtle effect
                    let value = (energy * 30.0) as u8; // Max 30 to keep it dark
                    Color::new(value, value, value, 255)
                } else {
                    Color::black()
                }
            },
        }
    }
}