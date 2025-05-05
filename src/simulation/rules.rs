/// Trait for different Game of Life rule sets
pub trait RuleSet {
    /// Apply rules to determine the next state of a cell
    fn apply(&self, current_state: bool, neighbors: u8) -> bool;
}

/// Standard Conway's Game of Life rules:
/// 1. Any live cell with fewer than two live neighbors dies (underpopulation)
/// 2. Any live cell with two or three live neighbors lives on to the next generation
/// 3. Any live cell with more than three live neighbors dies (overpopulation)
/// 4. Any dead cell with exactly three live neighbors becomes a live cell (reproduction)
pub struct StandardRuleSet;

impl StandardRuleSet {
    pub fn new() -> Self {
        StandardRuleSet
    }
}

impl RuleSet for StandardRuleSet {
    fn apply(&self, current_state: bool, neighbors: u8) -> bool {
        match (current_state, neighbors) {
            // Live cell with 2 or 3 neighbors survives
            (true, 2) | (true, 3) => true,
            // Dead cell with exactly 3 neighbors becomes alive
            (false, 3) => true,
            // All other cells die or stay dead
            _ => false,
        }
    }
}

/// Audio-driven rule set that modifies Conway's rules based on audio characteristics
pub struct AudioDrivenRuleSet {
    bass_energy: f32,
    mid_energy: f32,
    treble_energy: f32,
}

impl AudioDrivenRuleSet {
    pub fn new(bass: f32, mid: f32, treble: f32) -> Self {
        AudioDrivenRuleSet {
            bass_energy: bass.clamp(0.0, 1.0),
            mid_energy: mid.clamp(0.0, 1.0),
            treble_energy: treble.clamp(0.0, 1.0),
        }
    }
    
    /// Get the birth threshold based on bass energy
    /// Higher bass = more births (lower threshold)
    fn birth_threshold(&self) -> u8 {
        // Standard birth threshold is 3
        // Bass energy can lower this to 2 at max intensity
        if self.bass_energy > 0.8 {
            2 // High bass energy allows cells to be born with just 2 neighbors
        } else {
            3 // Standard rule
        }
    }
    
    /// Get survival range based on mid frequencies
    /// Mid frequencies affect survival rules
    fn survival_range(&self) -> (u8, u8) {
        // Standard survival range is [2, 3]
        // Mid frequencies can extend this range
        let lower = 2; // Always need at least 2 neighbors to survive
        
        // Higher mid energy can allow survival with more neighbors
        let upper = if self.mid_energy > 0.7 {
            4 // Allow survival with 4 neighbors at high mid energy
        } else if self.mid_energy > 0.4 {
            3 // Standard rule
        } else {
            // At very low mid energy, make survival harder
            // This causes more cell death during quiet parts
            2
        };
        
        (lower, upper)
    }
    
    /// Get mutation probability based on treble energy
    /// Higher treble = more random mutations
    fn mutation_chance(&self) -> f32 {
        // Treble energy directly influences mutation rate
        // Max mutation rate of 5% at highest treble
        self.treble_energy * 0.05
    }
} 
