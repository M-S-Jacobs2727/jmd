use super::{Grid, UpdateSettings};
use crate::{utils::distance_squared, Container};

/// Used for computing a list of neighboring particles
pub struct NeighborList {
    grid: Grid,
    force_distance: f64,
    skin_distance: f64,
    neighbor_distance: f64,
    stencil: Vec<[i32; 3]>,
    neighbors: Vec<Vec<usize>>,
    pub update_settings: UpdateSettings,
}
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

fn bin_atoms(grid: &Grid, positions: &Vec<[f64; 3]>) -> Vec<Vec<usize>> {
    let mut atom_indices_per_bin: Vec<Vec<usize>> = Vec::new();
    atom_indices_per_bin.resize(grid.total_num_bins(), Vec::new());
    positions
        .iter()
        .map(|p| {
            // dbg!(p);
            grid.coord_to_3d_idx(p)
        })
        .enumerate()
        .for_each(|(atom_idx, bin_idx)| {
            // dbg!(bin_idx);
            let lin_idx = grid.bin_idx_from_3d_idx(&bin_idx);
            // dbg!(lin_idx);
            atom_indices_per_bin[lin_idx].push(atom_idx)
        });
    atom_indices_per_bin
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
        let cutoff_distance = force_distance + skin_distance;
        let grid = Grid::new(container, bin_size, cutoff_distance);
        let stencil = compute_stencil(bin_size, cutoff_distance);
        dbg!(&grid);
        Self {
            grid,
            force_distance,
            skin_distance,
            neighbor_distance: cutoff_distance,
            stencil,
            neighbors,
            update_settings: UpdateSettings::new(1, 0, true),
        }
    }
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
        self.neighbor_distance.clone()
    }
    pub fn grid(&self) -> &Grid {
        &self.grid
    }
    pub fn update(&mut self, positions: &Vec<[f64; 3]>) {
        let num_atoms = positions.len();

        self.neighbors.clear();
        self.neighbors.resize(num_atoms, Vec::new());
        let neigh_dist_sq = self.neighbor_distance * self.neighbor_distance;

        // dbg!(positions);
        let atom_indices_per_bin = bin_atoms(&self.grid, &positions);
        for (i, pos) in positions.iter().enumerate() {
            let bin_idx = self.grid.coord_to_3d_idx(pos);
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
}
