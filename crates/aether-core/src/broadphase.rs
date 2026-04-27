//! # Broadphase Collision Detection
//!
//! φ-scaled spatial hash grid for O(1) average-case pair detection.
//! Cell size is governed by φ × max_body_extent for optimal distribution.

use crate::math::{Vec3, AABB};
use crate::phi;
use std::collections::HashMap;

/// A pair of body indices that might be colliding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BroadPair(pub u32, pub u32);

impl BroadPair {
    pub fn new(a: u32, b: u32) -> Self {
        if a < b { Self(a, b) } else { Self(b, a) }
    }
}

/// Grid cell coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellKey {
    x: i32,
    y: i32,
    z: i32,
}

/// φ-scaled spatial hash grid.
pub struct SpatialGrid {
    cell_size: f64,
    inv_cell_size: f64,
    cells: HashMap<CellKey, Vec<u32>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f64) -> Self {
        let cs = cell_size.max(0.01);
        Self {
            cell_size: cs,
            inv_cell_size: 1.0 / cs,
            cells: HashMap::with_capacity(256),
        }
    }

    /// Create with φ-scaled cell size based on the largest body extent
    pub fn phi_scaled(max_extent: f64) -> Self {
        Self::new(max_extent * phi::BROADPHASE_CELL_RATIO)
    }

    fn cell_key(&self, pos: Vec3) -> CellKey {
        CellKey {
            x: (pos.x * self.inv_cell_size).floor() as i32,
            y: (pos.y * self.inv_cell_size).floor() as i32,
            z: (pos.z * self.inv_cell_size).floor() as i32,
        }
    }

    /// Clear all cells for a new frame
    pub fn clear(&mut self) {
        for cell in self.cells.values_mut() {
            cell.clear();
        }
    }

    /// Insert a body's AABB into the grid
    pub fn insert(&mut self, index: u32, aabb: &AABB) {
        let min_key = self.cell_key(aabb.min);
        let max_key = self.cell_key(aabb.max);
        for x in min_key.x..=max_key.x {
            for y in min_key.y..=max_key.y {
                for z in min_key.z..=max_key.z {
                    let key = CellKey { x, y, z };
                    self.cells.entry(key).or_insert_with(|| Vec::with_capacity(4)).push(index);
                }
            }
        }
    }

    /// Collect all broad-phase collision pairs
    pub fn find_pairs(&self) -> Vec<BroadPair> {
        let mut pairs = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for cell in self.cells.values() {
            let n = cell.len();
            for i in 0..n {
                for j in (i + 1)..n {
                    let pair = BroadPair::new(cell[i], cell[j]);
                    if seen.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
        pairs
    }

    pub fn update_cell_size(&mut self, max_extent: f64) {
        let new_cs = max_extent * phi::BROADPHASE_CELL_RATIO;
        if (new_cs - self.cell_size).abs() > self.cell_size * 0.2 {
            self.cell_size = new_cs;
            self.inv_cell_size = 1.0 / new_cs;
            self.cells.clear();
        }
    }
}
