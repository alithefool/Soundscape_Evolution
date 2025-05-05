mod audio;
mod simulation;
mod renderer;
mod config;

use anyhow::{Result, Context};
use clap::Parser;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Sender, Receiver};

use crate::audio::player::AudioPlayer;
use crate::audio::analyzer::AudioAnalyzer;
use crate::simulation::gol::GameOfLife;
use crate::renderer::display::Display;
use crate::config::Config;

/// Soundscape Evolution - Conway's Game of Life visualizer driven by audio
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to audio file (MP3/WAV)
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Load configuration (either from file or use defaults)
    let config = match args.config {
        Some(path) => Config::from_file(&path)?,
        None => Config::default(),
    };
    let config = Arc::new(config);
    
    // Create channels for communication between audio and visualization
    let (audio_sender, audio_receiver) = bounded::<audio::analyzer::AudioFrame>(2);
    
    // Initialize components
    let mut player = AudioPlayer::new(config.clone())?;
    let analyzer = AudioAnalyzer::new(config.clone(), audio_sender);
    
    // Initialize game of life simulation
    let simulation = Arc::new(Mutex::new(
        GameOfLife::new(
            config.simulation.width,
            config.simulation.height,
            config.simulation.initial_seed,
        )
    ));

    // Initialize the display/renderer
    let mut display = Display::new(
        config.clone(),
        simulation.clone(),
        audio_receiver,
    )?;

    // If audio file was provided, load it
    if let Some(file_path) = args.file {
        player.load_file(&file_path)
            .context("Failed to load audio file")?;
        
        // Start the audio playback with analyzer callback
        player.play(analyzer)?;
    } else {
        println!("No audio file specified. Use --file to specify an audio file.");
        println!("Running with just the Game of Life simulation.");
    }

    // Run the display/renderer (this will block until the window is closed)
    display.run()?;

    Ok(())
}