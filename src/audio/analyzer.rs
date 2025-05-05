use anyhow::Result;
use crossbeam_channel::Sender;
use rustfft::{Fft, FftPlanner};
use std::sync::{Arc, Mutex};
use std::num::Complex;

use crate::config::Config;

/// Represents an analyzed audio frame with frequency band information
#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub bass_energy: f32,   // Energy in bass frequencies
    pub mid_energy: f32,    // Energy in mid frequencies
    pub treble_energy: f32, // Energy in treble frequencies
    pub peak_frequency: f32, // Most prominent frequency
    pub overall_energy: f32, // Overall audio energy
}

/// Analyzes audio data using FFT to extract frequency information
pub struct AudioAnalyzer {
    config: Arc<Config>,
    fft: Arc<dyn Fft<f32>>,
    sender: Sender<AudioFrame>,
    buffer: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
}

impl AudioAnalyzer {
    pub fn new(config: Arc<Config>, sender: Sender<AudioFrame>) -> Self {
        let fft_size = config.audio.fft_size;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let buffer = vec![Complex::new(0.0, 0.0); fft_size];
        let scratch = vec![Complex::new(0.0, 0.0); fft_size];
        
        AudioAnalyzer {
            config,
            fft,
            sender,
            buffer,
            scratch,
        }
    }

    /// Process a raw audio buffer and extract frequency information
    pub fn process_audio(&mut self, samples: &[f32]) -> Result<AudioFrame> {
        let fft_size = self.config.audio.fft_size;
        let sample_rate = self.config.audio.sample_rate as f32;
        
        // Prepare input buffer (apply window function and convert to complex)
        for i in 0..fft_size.min(samples.len()) {
            // Apply a simple Hann window function
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos());
            self.buffer[i] = Complex::new(samples[i] * window, 0.0);
        }
        
        // Zero-pad if needed
        for i in samples.len()..fft_size {
            self.buffer[i] = Complex::new(0.0, 0.0);
        }
        
        // Perform FFT
        self.fft.process_with_scratch(&mut self.buffer, &mut self.scratch);
        
        // Analyze frequency bands
        let bass_range = self.config.audio.bass_range;
        let mid_range = self.config.audio.mid_range;
        let treble_range = self.config.audio.treble_range;
        
        let bin_width = sample_rate / fft_size as f32;
        
        let bass_start = (bass_range.0 / bin_width) as usize;
        let bass_end = (bass_range.1 / bin_width) as usize;
        
        let mid_start = (mid_range.0 / bin_width) as usize;
        let mid_end = (mid_range.1 / bin_width) as usize;
        
        let treble_start = (treble_range.0 / bin_width) as usize;
        let treble_end = (treble_range.1 / bin_width) as usize;
        
        // Calculate energy in each band
        let bass_energy = self.calculate_band_energy(bass_start, bass_end);
        let mid_energy = self.calculate_band_energy(mid_start, mid_end);
        let treble_energy = self.calculate_band_energy(treble_start, treble_end);
        
        // Find peak frequency
        let mut max_magnitude = 0.0;
        let mut peak_bin = 0;
        
        for bin in 1..fft_size / 2 {
            let magnitude = self.buffer[bin].norm();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                peak_bin = bin;
            }
        }
        
        let peak_frequency = peak_bin as f32 * bin_width;
        let overall_energy = bass_energy + mid_energy + treble_energy;
        
        // Apply sensitivity adjustment
        let sensitivity = self.config.audio.sensitivity;
        let frame = AudioFrame {
            bass_energy: bass_energy * sensitivity,
            mid_energy: mid_energy * sensitivity,
            treble_energy: treble_energy * sensitivity,
            peak_frequency,
            overall_energy: overall_energy * sensitivity,
        };
        
        // Send the frame to the visualization thread
        let _ = self.sender.try_send(frame.clone());
        
        Ok(frame)
    }
    
    fn calculate_band_energy(&self, start_bin: usize, end_bin: usize) -> f32 {
        let mut energy = 0.0;
        
        // Use only the first half of the FFT output (the rest is mirrored)
        let bins = self.buffer.len() / 2;
        
        let start = start_bin.clamp(1, bins); // Skip DC bin
        let end = end_bin.clamp(1, bins);
        
        for bin in start..end {
            // Magnitude squared is proportional to energy
            energy += self.buffer[bin].norm_sqr();
        }
        
        // Normalize by band width
        if end > start {
            energy /= (end - start) as f32;
        }
        
        energy.sqrt() // Convert to amplitude
    }
}

// For testing/development without real audio input
impl AudioAnalyzer {
    pub fn generate_test_frame(&self, time: f32) -> AudioFrame {
        // Generate synthetic audio response for testing
        let bass = (time * 2.0).sin() * 0.5 + 0.5;
        let mid = (time * 3.0).sin() * 0.5 + 0.5;
        let treble = (time * 5.0).sin() * 0.5 + 0.5;
        
        AudioFrame {
            bass_energy: bass,
            mid_energy: mid,
            treble_energy: treble,
            peak_frequency: 440.0, // A4 note
            overall_energy: (bass + mid + treble) / 3.0,
        }
    }
}