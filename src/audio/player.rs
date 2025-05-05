use anyhow::{Result, Context};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::io::Cursor;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::audio::analyzer::AudioAnalyzer;
use crate::config::Config;

/// Handles audio file loading and playback
pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
    config: Arc<Config>,
}

impl AudioPlayer {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .context("Failed to initialize audio output stream")?;
        
        let sink = Sink::try_new(&stream_handle)
            .context("Failed to create audio sink")?;
            
        Ok(AudioPlayer {
            _stream: stream,
            sink,
            config,
        })
    }
    
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // Clear any existing audio
        self.sink.clear();
        
        let file = File::open(path.as_ref())
            .context("Failed to open audio file")?;
        
        let source = Decoder::new(BufReader::new(file))
            .context("Failed to decode audio file")?;
            
        // Prepare the audio source
        self.sink.append(source);
        self.sink.pause(); // Start paused so we can synchronize with the visualization
        
        Ok(())
    }
    
    pub fn play(&mut self, analyzer: AudioAnalyzer) -> Result<()> {
        // Create a media source from the file
        // Note: In a real implementation, we'd need to set up audio interceptors 
        // to feed the analyzer in real-time. This is simplified for the example.
        
        // Set up audio analysis
        // In a full implementation, we'd intercept the audio stream and pass frames to analyzer
        
        // For now, just start playback
        self.sink.play();
        
        Ok(())
    }
    
    pub fn pause(&mut self) {
        self.sink.pause();
    }
    
    pub fn stop(&mut self) {
        self.sink.stop();
    }
    
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
    }
    
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }
}