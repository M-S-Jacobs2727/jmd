use super::{Grid, UpdateSettings};
use crate::{
    utils::{computations::distance_squared, indices::Index},
    Container,
};

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
#[derive(Debug)]
pub struct NeighborList {
    grid: Grid,
    force_distance: f64,
    skin_distance: f64,
    stencil: Vec<[i32; 3]>,
    neighbors: Vec<Vec<usize>>,
    is_built: bool,
    pub update_settings: UpdateSettings,
}
impl NeighborList {
    pub fn new(
        container: &Container,
        force_distance: f64,
        skin_distance: f64,
        update_settings: UpdateSettings,
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
        let bin_size = cutoff_distance * 0.5;
        let grid = Grid::new(container, bin_size, cutoff_distance);
        let stencil = compute_stencil(bin_size, cutoff_distance);
        // dbg!(&grid);
        Self {
            grid,
            force_distance,
            skin_distance,
            stencil,
            neighbors,
            is_built: false,
            update_settings,
        }
    }

    // Getters
    pub fn neighbors(&self) -> &Vec<Vec<usize>> {
        &self.neighbors
    }
    pub fn force_distance(&self) -> f64 {
        self.force_distance
    }
    pub fn skin_distance(&self) -> f64 {
        self.skin_distance
    }
    pub fn neighbor_distance(&self) -> f64 {
        self.skin_distance + self.force_distance
    }
    pub fn grid(&self) -> &Grid {
        &self.grid
    }
    pub fn update_settigs(&self) -> &UpdateSettings {
        &self.update_settings
    }
    pub fn is_built(&self) -> bool {
        self.is_built
    }

    // Setters
    pub fn set_grid(&mut self, grid: Grid) {
        self.is_built = false;
        self.grid = grid
    }
    pub fn set_skin_distance(&mut self, skin_distance: f64) {
        if skin_distance <= 0.0 {
            panic!("Skin distance must be positive, found {}", skin_distance);
        }
        self.is_built = false;
        self.skin_distance = skin_distance;
    }
    pub fn set_update_settings(&mut self, update_settings: UpdateSettings) {
        self.update_settings = update_settings
    }

    pub fn update(&mut self, positions: &Vec<[f64; 3]>) {
        self.is_built = true;
        let num_atoms = positions.len();

        self.neighbors.clear();
        self.neighbors.resize(num_atoms, Vec::new());
        let neigh_dist_sq = self.neighbor_distance() * self.neighbor_distance();

        // dbg!(positions);
        let atom_indices_per_bin = self.bin_atoms(&positions);
        positions.iter().enumerate().for_each(|(i, pos)| {
            let bin_idx = self.grid.coord_to_index(pos);
            // dbg!(pos);
            // dbg!(&bin_idx);
            let bin_3d = bin_idx.to_3d();
            for offset in &self.stencil {
                let comp_bin = Index::from_3d(
                    &[
                        (offset[0] + bin_3d[0] as i32) as usize,
                        (offset[1] + bin_3d[1] as i32) as usize,
                        (offset[2] + bin_3d[2] as i32) as usize,
                    ],
                    &self.grid.num_bins(),
                );
                for &neigh_idx in &atom_indices_per_bin[comp_bin.idx()] {
                    if distance_squared(&positions[neigh_idx], pos) < neigh_dist_sq {
                        self.neighbors[i].push(neigh_idx);
                    }
                }
            }
        });
    }
    fn bin_atoms(&self, positions: &Vec<[f64; 3]>) -> Vec<Vec<usize>> {
        let mut atom_indices_per_bin: Vec<Vec<usize>> = Vec::new();
        atom_indices_per_bin.resize(self.grid.total_num_bins(), Vec::new());
        positions
            .iter()
            .map(|p| {
                // dbg!(p);
                self.grid.coord_to_index(p)
            })
            .enumerate()
            .for_each(|(atom_idx, bin_idx)| {
                // dbg!(bin_idx);
                atom_indices_per_bin[bin_idx.idx()].push(atom_idx)
            });
        atom_indices_per_bin
    }
}
