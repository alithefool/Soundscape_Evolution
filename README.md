# Soundscape Evolution

A native Linux media player that visualizes audio using Conway's Game of Life.

## Features

- Play local audio files (MP3/WAV)
- Visualize audio with a real-time Game of Life simulation
- Audio influences the simulation:
  - Bass frequencies increase cell birth rates
  - Mid frequencies modify survival rules
  - Treble frequencies influence colors and random mutations
- Multiple color schemes
- Fullscreen support
- Keyboard controls

## Requirements

- Rust (latest stable version recommended)
- Linux with audio and graphics support
- Required system packages:
  - alsa-lib-devel (for audio)
  - libX11-devel, libXi-devel, libXrandr-devel (for windowing)

## Installation

1. Clone this repository:
   ```
   git clone https://github.com/yourusername/soundscape-evolution.git
   cd soundscape-evolution
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the application:
   ```
   ./target/release/soundscape_evolution --file /path/to/your/audio.mp3
   ```

## Usage

### Command Line Arguments

- `--file` or `-f`: Path to audio file (MP3/WAV)
- `--config` or `-c`: Path to custom configuration file (TOML)

### Keyboard Controls

- `Esc`: Toggle fullscreen
- `Space`: Reset simulation with random cells
- `C`: Clear the simulation
- `1-4`: Switch color schemes
  - `1`: Classic (Black & White)
  - `2`: Heat Map
  - `3`: Rainbow
  - `4`: Pulse (Audio Reactive)

## Configuration

You can customize the application by creating a `config.toml` file. Here's an example:

```toml
[window]
title = "Soundscape Evolution"
width = 800
height = 600
fullscreen = false

[audio]
sample_rate = 44100
channels = 2
fft_size = 2048
bass_range = [20.0, 250.0]
mid_range = [250.0, 2000.0]
treble_range = [2000.0, 20000.0]
sensitivity = 1.0

[simulation]
width = 200
height = 150
update_rate = 30.0
initial_seed = 0.3
edge_behavior = "Wrap"  # "Wrap", "Dead", or "Alive"

[visualization]
cell_size = 4
color_scheme = "Pulse"  # "Classic", "Heat", "Rainbow", or "Pulse"
fade_rate = 0.1
```

## Development

The project is structured as follows:

```
soundscape_evolution/
├── src/
│   ├── main.rs           # Application entry point
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── player.rs     # Audio playback
│   │   ├── analyzer.rs   # FFT + frequency band analysis
│   ├── simulation/
│   │   ├── mod.rs
│   │   ├── gol.rs        # Game of Life engine
│   │   ├── rules.rs      # Audio-driven rule modifiers
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── display.rs    # Renders grid to window
│   │   ├── color.rs      # Color schemes / dynamic visuals
│   └── config.rs         # Global config constants
├── Cargo.toml
└── README.md
```

## Technical Details

- **Audio Analysis**: Uses FFT to extract frequency bands from audio
- **Game of Life**: Conway's Game of Life with audio-modifiable rules
- **Rendering**: Efficient pixel-based rendering with minimal overhead
- **Communication**: Thread-safe channels for passing audio data to visualization

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Using [rodio](https://github.com/RustAudio/rodio) for audio playback
- [pixels](https://github.com/parasyte/pixels) for efficient 2D rendering
- [winit](https://github.com/rust-windowing/winit) for cross-platform windowing