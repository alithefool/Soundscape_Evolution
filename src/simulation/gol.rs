use rand::Rng;
use std::time::Instant;

use crate::audio::analyzer::AudioFrame;
use crate::simulation::rules::{RuleSet, StandardRuleSet, AudioDrivenRuleSet};
use crate::config::EdgeBehavior;

/// The core Game of Life simulation
pub struct GameOfLife {
    width: usize,
    height: usize,
    grid: Vec<bool>,      // Current state
    next_grid: Vec<bool>, // Next state
    age_grid: Vec<u8>,    // How many generations a cell has been alive
    last_update: Instant,
    ruleset: Box<dyn RuleSet>,
    edge_behavior: EdgeBehavior,
}

impl GameOfLife {
    pub fn new(width: usize, height: usize, initial_density: f32) -> Self {
        let cell_count = width * height;
        let mut rng = rand::thread_rng();
        
        // Initialize grid with random cells
        let mut grid = vec![false; cell_count];
        for cell in grid.iter_mut() {
            *cell = rng.gen::<f32>() < initial_density;
        }
        
        let next_grid = vec![false; cell_count];
        let age_grid = vec![0; cell_count];
        
        GameOfLife {
            width,
            height,
            grid,
            next_grid,
            age_grid,
            last_update: Instant::now(),
            ruleset: Box::new(StandardRuleSet::new()),
            edge_behavior: EdgeBehavior::Wrap,
        }
    }
    
    /// Update the simulation with potential audio influence
    pub fn update(&mut self, audio_frame: Option<&AudioFrame>) {
        // If we have audio data, use it to affect the rules
        if let Some(frame) = audio_frame {
            let ruleset = AudioDrivenRuleSet::new(
                frame.bass_energy,
                frame.mid_energy,
                frame.treble_energy,
            );
            self.ruleset = Box::new(ruleset);
        }

        // Apply rules to calculate the next generation
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let neighbors = self.count_neighbors(x, y);
                let current_state = self.grid[idx];
                
                // Apply the ruleset to determine the next state
                let next_state = self.ruleset.apply(current_state, neighbors);
                
                self.next_grid[idx] = next_state;
                
                // Update cell age
                if next_state {
                    if current_state {
                        // Cell stays alive, increment age (with saturation)
                        self.age_grid[idx] = self.age_grid[idx].saturating_add(1);
                    } else {
                        // Cell was born, reset age
                        self.age_grid[idx] = 1;
                    }
                } else {
                    // Cell is dead, reset age
                    self.age_grid[idx] = 0;
                }
            }
        }
        
        // Swap grids for next iteration
        std::mem::swap(&mut self.grid, &mut self.next_grid);
        self.last_update = Instant::now();
    }
    
    /// Count the number of live neighbors for a cell
    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                // Skip the center cell (self)
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = self.wrap_x(x as isize + dx);
                let ny = self.wrap_y(y as isize + dy);
                
                if let Some(idx) = self.get_index(nx, ny) {
                    if self.grid[idx] {
                        count += 1;
                    }
                } else {
                    // Handle edge behavior for out-of-bounds cells
                    match self.edge_behavior {
                        EdgeBehavior::Wrap => {}, // Already handled by wrap_x/y
                        EdgeBehavior::Dead => {}, // Count as dead, do nothing
                        EdgeBehavior::Alive => count += 1, // Count as alive
                    }
                }
            }
        }
        
        count
    }
    
    /// Wrap x-coordinate based on edge behavior
    fn wrap_x(&self, x: isize) -> isize {
        match self.edge_behavior {
            EdgeBehavior::Wrap => {
                if x < 0 {
                    (self.width as isize + x) % self.width as isize
                } else {
                    x % self.width as isize
                }
            },
            _ => x, // For Dead/Alive edge behaviors, don't wrap
        }
    }
    
    /// Wrap y-coordinate based on edge behavior
    fn wrap_y(&self, y: isize) -> isize {
        match self.edge_behavior {
            EdgeBehavior::Wrap => {
                if y < 0 {
                    (self.height as isize + y) % self.height as isize
                } else {
                    y % self.height as isize
                }
            },
            _ => y, // For Dead/Alive edge behaviors, don't wrap
        }
    }
    
    /// Convert x,y coordinates to grid index if valid
    fn get_index(&self, x: isize, y: isize) -> Option<usize> {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            Some(y as usize * self.width + x as usize)
        } else {
            None
        }
    }
    
    /// Set specific cell state
    pub fn set_cell(&mut self, x: usize, y: usize, alive: bool) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.grid[idx] = alive;
        }
    }
    
    /// Get the state of a specific cell
    pub fn is_cell_alive(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.grid[idx]
        } else {
            false
        }
    }
    
    /// Get the age of a specific cell
    pub fn cell_age(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.age_grid[idx]
        } else {
            0
        }
    }
    
    /// Get simulation width
    pub fn width(&self) -> usize {
        self.width
    }
    
    /// Get simulation height
    pub fn height(&self) -> usize {
        self.height
    }
    
    /// Clear the grid (all cells dead)
    pub fn clear(&mut self) {
        for cell in self.grid.iter_mut() {
            *cell = false;
        }
        for age in self.age_grid.iter_mut() {
            *age = 0;
        }
    }
    
    /// Randomize the grid with a specified density
    pub fn randomize(&mut self, density: f32) {
        let mut rng = rand::thread_rng();
        for cell in self.grid.iter_mut() {
            *cell = rng.gen::<f32>() < density;
        }
        for age in self.age_grid.iter_mut() {
            *age = 0;
        }
    }
    
    /// Set edge behavior
    pub fn set_edge_behavior(&mut self, behavior: EdgeBehavior) {
        self.edge_behavior = behavior;
    }
}