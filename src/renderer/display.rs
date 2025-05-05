use anyhow::{Result, Context};
use crossbeam_channel::Receiver;
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder, Fullscreen};

use crate::audio::analyzer::AudioFrame;
use crate::config::{Config, ColorScheme};
use crate::renderer::color::ColorPalette;
use crate::simulation::gol::GameOfLife;

pub struct Display {
    config: Arc<Config>,
    simulation: Arc<Mutex<GameOfLife>>,
    audio_receiver: Receiver<AudioFrame>,
    color_palette: ColorPalette,
    last_frame_time: Instant,
    last_sim_update: Instant,
    current_audio_frame: Option<AudioFrame>,
}

impl Display {
    pub fn new(
        config: Arc<Config>,
        simulation: Arc<Mutex<GameOfLife>>,
        audio_receiver: Receiver<AudioFrame>,
    ) -> Result<Self> {
        let color_palette = ColorPalette::new(config.visualization.color_scheme.clone());
        
        Ok(Display {
            config,
            simulation,
            audio_receiver,
            color_palette,
            last_frame_time: Instant::now(),
            last_sim_update: Instant::now(),
            current_audio_frame: None,
        })
    }
    
    pub fn run(&mut self) -> Result<()> {
        let event_loop = EventLoop::new();
        
        // Create window
        let window_width = self.config.window.width;
        let window_height = self.config.window.height;
        
        let window = WindowBuilder::new()
            .with_title(&self.config.window.title)
            .with_inner_size(LogicalSize::new(window_width, window_height))
            .with_resizable(true)
            .build(&event_loop)
            .context("Failed to create window")?;
            
        // Set fullscreen if configured
        if self.config.window.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        
        // Create pixel buffer
        let surface_texture = SurfaceTexture::new(window_width, window_height, &window);
        let mut pixels = Pixels::new(window_width, window_height, surface_texture)
            .context("Failed to create pixel buffer")?;
            
        // Main event loop
        let _ = event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::Resized(new_size) => {
                        // Resize pixel buffer
                        if pixels.resize_surface(new_size.width, new_size.height).is_err() {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                    },
                    WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(key), state: ElementState::Pressed, .. }, .. } => {
                        self.handle_keyboard_input(key, &window, &mut pixels);
                    },
                    _ => {},
                },
                Event::MainEventsCleared => {
                    // Check for new audio data
                    while let Ok(frame) = self.audio_receiver.try_recv() {
                        self.current_audio_frame = Some(frame);
                    }
                    
                    // Update simulation at fixed rate
                    let now = Instant::now();
                    let sim_delta = now.duration_since(self.last_sim_update).as_secs_f32();
                    
                    if sim_delta >= 1.0 / self.config.simulation.update_rate {
                        if let Ok(mut sim) = self.simulation.lock() {
                            sim.update(self.current_audio_frame.as_ref());
                            self.last_sim_update = now;
                        }
                    }
                    
                    // Calculate frame time for animations
                    let frame_delta = now.duration_since(self.last_frame_time).as_secs_f32();
                    self.last_frame_time = now;
                    
                    // Update color palette
                    self.color_palette.update(self.current_audio_frame.as_ref(), frame_delta);
                    
                    // Render frame
                    self.render(pixels.frame_mut());
                    
                    if pixels.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    
                    // Aim for ~60 FPS for the visualization
                    window.request_redraw();
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + Duration::from_millis(16)
                    );
                },
                _ => {},
            }
        });
    }
    
    fn render(&self, frame: &mut [u8]) {
        let background_color = self.color_palette.get_background_color();
        
        // Only acquire lock once to minimize contention
        if let Ok(sim) = self.simulation.lock() {
            let width = sim.width();
            let height = sim.height();
            let cell_size = self.config.visualization.cell_size;
            
            // Clear frame with background color
            for pixel in frame.chunks_exact_mut(4) {
                pixel.copy_from_slice(&background_color.to_rgba());
            }
            
            // Render cells
            let window_width = self.config.window.width as usize;
            
            for y in 0..height {
                for x in 0..width {
                    let age = sim.cell_age(x, y);
                    if age > 0 {
                        let cell_color = self.color_palette.get_cell_color(age, 255);
                        
                        // Draw cell rectangle
                        for cy in 0..cell_size {
                            for cx in 0..cell_size {
                                let px = x * cell_size as usize + cx as usize;
                                let py = y * cell_size as usize + cy as usize;
                                
                                // Check if within window bounds
                                if px < window_width && py < self.config.window.height as usize {
                                    let idx = (py * window_width + px) * 4;
                                    if idx + 3 < frame.len() {
                                        frame[idx..idx + 4].copy_from_slice(&cell_color.to_rgba());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn handle_keyboard_input(&mut self, key: VirtualKeyCode, window: &winit::window::Window, pixels: &mut Pixels) {
        match key {
            VirtualKeyCode::Escape => {
                // Toggle fullscreen
                let is_fullscreen = window.fullscreen().is_some();
                window.set_fullscreen(if is_fullscreen {
                    None
                } else {
                    Some(Fullscreen::Borderless(None))
                });
            },
            VirtualKeyCode::Space => {
                // Reset simulation with random state
                if let Ok(mut sim) = self.simulation.lock() {
                    sim.randomize(self.config.simulation.initial_seed);
                }
            },
            VirtualKeyCode::C => {
                // Clear simulation
                if let Ok(mut sim) = self.simulation.lock() {
                    sim.clear();
                }
            },
            VirtualKeyCode::Key1 => {
                // Switch to Classic color scheme
                self.color_palette.set_scheme(ColorScheme::Classic);
            },
            VirtualKeyCode::Key2 => {
                // Switch to Heat color scheme
                self.color_palette.set_scheme(ColorScheme::Heat);
            },
            VirtualKeyCode::Key3 => {
                // Switch to Rainbow color scheme
                self.color_palette.set_scheme(ColorScheme::Rainbow);
            },
            VirtualKeyCode::Key4 => {
                // Switch to Pulse color scheme
                self.color_palette.set_scheme(ColorScheme::Pulse);
            },
            _ => {},
        }
    }
}