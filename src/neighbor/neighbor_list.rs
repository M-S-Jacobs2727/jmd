use std::rc::Rc;

use super::Grid;
use crate::{
    container::Container,
    utils::{computations::distance_squared, Index},
};

/// Used for computing a list of neighboring particles
#[derive(Debug)]
pub struct NeighborList {
    grid: Grid,
    stencil: Vec<[i32; 3]>,
    neighbors: Vec<Vec<usize>>,
    force_distance: f64,
    skin_distance: f64,
}
impl NeighborList {
    pub fn new(container: Rc<Container>, force_distance: f64, skin_distance: f64) -> Self {
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
        let neighbor_distance = skin_distance + force_distance;
        let bin_size = neighbor_distance * 0.5;
        let stencil = NeighborList::compute_stencil(bin_size, neighbor_distance);
        let grid = Grid::new(container, bin_size, neighbor_distance);
        Self {
            grid,
            stencil,
            neighbors: Vec::new(),
            force_distance,
            skin_distance,
        }
    }
    /// Compute a set of integer offsets to a bin index that corresponds
    /// to populating a half neighbor list
    fn compute_stencil(bin_size: f64, neighbor_distance: f64) -> Vec<[i32; 3]> {
        let max_number_out = (neighbor_distance / bin_size).ceil() as i32;
        let mut stencil: Vec<[i32; 3]> = Vec::new();
        for i in 0..max_number_out + 1 {
            stencil.push([i, 0, 0]);
        }
        for i in -max_number_out..max_number_out + 1 {
            for j in 1..max_number_out + 1 {
                let i2 = (i.abs() - 1).max(0);
                let j2 = (j.abs() - 1).max(0);
                let min_dist = ((i2 * i2 + j2 * j2) as f64).sqrt();
                if min_dist < neighbor_distance {
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
                    if min_dist < neighbor_distance {
                        stencil.push([i, j, k]);
                    }
                }
            }
        }
        stencil
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
    pub fn max_neighbor_distance(&self) -> f64 {
        self.skin_distance + self.force_distance
    }
    pub fn is_built(&self) -> bool {
        !self.neighbors.is_empty()
    }

    // Setters
    pub fn set_bin_size(&mut self, bin_size: f64) {
        self.grid.set_bin_size(bin_size);
        self.stencil = NeighborList::compute_stencil(bin_size, self.max_neighbor_distance());
    }
    pub fn set_skin_distance(&mut self, skin_distance: f64) {
        if skin_distance <= 0.0 {
            panic!("Skin distance must be positive, found {}", skin_distance);
        }
        self.skin_distance = skin_distance;
        self.neighbors.clear();
        let bin_size = self.grid.bin_size();
        self.grid
            .set_neighbor_distance(self.max_neighbor_distance());
        self.stencil = NeighborList::compute_stencil(bin_size, self.max_neighbor_distance());
    }
    pub(crate) fn set_force_distance(&mut self, force_distance: f64) {
        self.force_distance = force_distance;
        self.neighbors.clear();
        let bin_size = self.grid.bin_size();
        self.grid
            .set_neighbor_distance(self.max_neighbor_distance());
        self.stencil = NeighborList::compute_stencil(bin_size, self.max_neighbor_distance());
    }

    /// Update the neighbor list based on the positions of the owned and ghost atoms in the current process
    pub fn update(&mut self, positions: &Vec<[f64; 3]>) {
        let num_atoms = positions.len();

        self.neighbors.clear();
        self.neighbors.resize(num_atoms, Vec::new());
        let neigh_dist_sq = self.max_neighbor_distance() * self.max_neighbor_distance();

        let atom_indices_per_bin = self.bin_atoms(&positions);
        positions.iter().enumerate().for_each(|(i, pos)| {
            let bin_idx = self.grid.coord_to_index(pos);
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
                    if distance_squared(&positions[neigh_idx], pos) < neigh_dist_sq
                        && neigh_idx != i
                    {
                        self.neighbors[i].push(neigh_idx);
                    }
                }
            }
        });
    }
    /// Assign each atom to a bin in the grid based on its position
    fn bin_atoms(&self, positions: &Vec<[f64; 3]>) -> Vec<Vec<usize>> {
        let mut atom_indices_per_bin: Vec<Vec<usize>> = Vec::new();
        atom_indices_per_bin.resize(self.grid.total_num_bins(), Vec::new());
        positions
            .iter()
            .map(|p| self.grid.coord_to_index(p))
            .enumerate()
            .for_each(|(atom_idx, bin_idx)| atom_indices_per_bin[bin_idx.idx()].push(atom_idx));
        atom_indices_per_bin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::BC;

    fn setup_nl() -> NeighborList {
        let container = Container::new(0.0, 10.0, 0.0, 10.0, 0.0, 10.0, BC::PP, BC::PP, BC::PP);
        NeighborList::new(Rc::new(container), 2.0, 1.0)
    }

    #[test]
    fn test_single_atom() {
        let mut nl = setup_nl();
        nl.update(&vec![[1.0, 1.0, 1.0]]);
        assert_eq!(nl.neighbors()[0], vec![]);
    }

    #[test]
    fn test_two_atoms() {
        let mut nl = setup_nl();
        nl.update(&vec![[1.0, 1.0, 1.0], [1.0, 1.0, 2.0]]);
        let neighbors = nl.neighbors();
        assert_eq!(neighbors[0], vec![1]);
        assert_eq!(neighbors[1], vec![]); // half neighbor list
    }

    #[test]
    fn test_two_atoms_far() {
        let mut nl = setup_nl();
        nl.update(&vec![[1.0, 1.0, 1.0], [1.0, 1.0, 9.0]]);
        let neighbors = nl.neighbors();
        assert_eq!(neighbors[0], vec![]);
        assert_eq!(neighbors[1], vec![]);
    }

    #[test]
    fn test_four_atoms() {
        let mut nl = setup_nl();
        let pos = vec![
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 9.0],
            [1.0, 2.0, 1.0],
            [1.0, 3.0, 9.0],
        ];
        dbg!(&nl.grid);

        nl.update(&pos);
        let neighbors = nl.neighbors();
        assert_eq!(neighbors[0], vec![2]);
        assert_eq!(neighbors[1], vec![3]);
        assert_eq!(neighbors[2], vec![]);
        assert_eq!(neighbors[3], vec![]);

        let bins = nl.bin_atoms(&pos);
        let filled_bins: Vec<(usize, &Vec<usize>)> = bins
            .iter()
            .enumerate()
            .filter(|(_i, b)| !b.is_empty())
            .collect();
        dbg!(&filled_bins);
        let occupied_bins = vec![
            (3usize * 121 + 3 * 11 + 3, 0usize),
            (3usize * 121 + 3 * 11 + 7, 1usize),
            (3usize * 121 + 3 * 11 + 3, 2usize),
            (3usize * 121 + 3 * 11 + 7, 3usize),
        ];
        bins.iter().enumerate().for_each(|(i, b)| {
            let res = occupied_bins.iter().find(|(j, _idx)| i == *j);
            let v = match res {
                Some((_j, idx)) => vec![*idx],
                None => vec![],
            };
            assert_eq!(v, *b);
        });
    }
}
