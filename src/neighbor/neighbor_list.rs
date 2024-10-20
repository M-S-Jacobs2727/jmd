use super::{Grid, UpdateSettings};
use crate::{utils::computations::distance_squared, Container};

fn compute_stencil(bin_size: f64, cutoff_distance: f64) -> Vec<[i32; 3]> {
    let max_number_out = (cutoff_distance / bin_size).ceil() as i32;
    let mut stencil: Vec<[i32; 3]> = Vec::new();
    for i in 0..max_number_out + 1 {
        stencil.push([i, 0, 0]);
    }
    for i in -max_number_out..max_number_out + 1 {
        for j in 1..max_number_out + 1 {
            let i2 = (i.abs() - 1).max(0);
            let j2 = (j.abs() - 1).max(0);
            let min_dist = ((i2 * i2 + j2 * j2) as f64).sqrt();
            if min_dist < cutoff_distance {
                stencil.push([i, j, 0]);
            }
        }
    }
    for i in -max_number_out..max_number_out + 1 {
        for j in -max_number_out..max_number_out + 1 {
            for k in 1..max_number_out + 1 {
                let i2 = (i.abs() - 1).max(0);
                let j2 = (j.abs() - 1).max(0);
                let k2 = (k.abs() - 1).max(0);
                let min_dist = ((i2 * i2 + j2 * j2 + k2 * k2) as f64).sqrt();
                if min_dist < cutoff_distance {
                    stencil.push([i, j, k]);
                }
            }
        }
    }
    stencil
}

/// Used for computing a list of neighboring particles
pub struct NeighborList {
    grid: Grid,
    force_distance: f64,
    skin_distance: f64,
    stencil: Vec<[i32; 3]>,
    neighbors: Vec<Vec<usize>>,
    pub update_settings: UpdateSettings,
}
impl NeighborList {
    pub fn new(
        container: &Container,
        bin_size: f64,
        force_distance: f64,
        skin_distance: f64,
    ) -> Self {
        let neighbors: Vec<Vec<usize>> = Vec::new();
        assert!(
            force_distance > 0.0,
            "Force cutoff distance ({}) must be positive",
            force_distance
        );
        assert!(
            skin_distance > 0.0,
            "Neighbor skin distance ({}) must be positive",
            skin_distance
        );
        let cutoff_distance = skin_distance + force_distance;
        let grid = Grid::new(container, bin_size, cutoff_distance);
        let stencil = compute_stencil(bin_size, cutoff_distance);
        dbg!(&grid);
        Self {
            grid,
            force_distance,
            skin_distance,
            stencil,
            neighbors,
            update_settings: UpdateSettings::new(1, 0, true),
        }
    }

    // Getters
    pub fn neighbors(&self) -> &Vec<Vec<usize>> {
        &self.neighbors
    }
    pub fn force_distance(&self) -> f64 {
        self.force_distance.clone()
    }
    pub fn skin_distance(&self) -> f64 {
        self.skin_distance.clone()
    }
    pub fn neighbor_distance(&self) -> f64 {
        self.skin_distance() + self.force_distance()
    }
    pub fn grid(&self) -> &Grid {
        &self.grid
    }
    pub fn update_settigs(&self) -> &UpdateSettings {
        &self.update_settings
    }

    // Setters
    pub fn set_grid(&mut self, grid: Grid) {
        self.grid = grid
    }
    pub fn set_skin_distance(&mut self, skin_distance: f64) {
        if skin_distance <= 0.0 {
            panic!("Skin distance must be positive, found {}", skin_distance);
        }
        self.skin_distance = skin_distance;
    }
    pub fn set_update_settings(&mut self, update_settings: UpdateSettings) {
        self.update_settings = update_settings
    }

    pub fn update(&mut self, positions: &Vec<[f64; 3]>) {
        let num_atoms = positions.len();

        self.neighbors.clear();
        self.neighbors.resize(num_atoms, Vec::new());
        let neigh_dist_sq = self.neighbor_distance() * self.neighbor_distance();

        // dbg!(positions);
        let atom_indices_per_bin = self.bin_atoms(&positions);
        for (i, pos) in positions.iter().enumerate() {
            let bin_idx = self.grid.coords_to_linear_indices(pos);
            // dbg!(&bin_idx);
            for offset in &self.stencil {
                let comp_bin = [
                    offset[0] + bin_idx[0],
                    offset[1] + bin_idx[1],
                    offset[2] + bin_idx[2],
                ];
                let comp_bin_linear = self.grid.bin_idx_from_3d_idx(&comp_bin);
                for &neigh_idx in &atom_indices_per_bin[comp_bin_linear] {
                    if distance_squared(&positions[neigh_idx], pos) < neigh_dist_sq {
                        self.neighbors[i].push(neigh_idx);
                    }
                }
            }
        }
    }
    fn bin_atoms(&self, positions: &Vec<[f64; 3]>) -> Vec<Vec<usize>> {
        let mut atom_indices_per_bin: Vec<Vec<usize>> = Vec::new();
        atom_indices_per_bin.resize(self.grid.total_num_bins(), Vec::new());
        positions
            .iter()
            .map(|p| {
                // dbg!(p);
                self.grid.coords_to_linear_indices(p)
            })
            .enumerate()
            .for_each(|(atom_idx, bin_idx)| {
                // dbg!(bin_idx);
                let lin_idx = self.grid.bin_idx_from_3d_idx(&bin_idx);
                // dbg!(lin_idx);
                atom_indices_per_bin[lin_idx].push(atom_idx)
            });
        atom_indices_per_bin
    }
}
