use crate::{atom::Atom, Domain};

pub struct NeighborListSettings {
    pub cell_size: f64,
    pub cutoff: f64,
    pub skin: f64,
    pub update_every: usize,
    pub delay_update: usize,
}
impl NeighborListSettings {
    pub fn new(cell_size: f64, cutoff: f64, update_every: usize, delay_update: usize) -> Self {
        Self {
            cell_size,
            cutoff,
            skin: 0.0,
            update_every,
            delay_update,
        }
    }
    pub fn neighbor_list(self, domain: &Domain) -> NeighborList {
        let mut neighbor_list = NeighborList::new(self);
        neighbor_list.set_stencil(&domain);
        neighbor_list
    }
}

pub struct NeighborList {
    pub cells: Vec<Vec<usize>>,
    pub neighbors: Vec<Vec<usize>>,
    pub settings: NeighborListSettings,
    pub cells_per_axis: [usize; 3],
    pub cell_origin: [f64; 3],
    stencil: Vec<[i64; 3]>,
}

impl NeighborList {
    pub fn new(settings: NeighborListSettings) -> Self {
        Self {
            cells: vec![],
            neighbors: vec![],
            settings,
            stencil: vec![],
            cells_per_axis: [0; 3],
            cell_origin: [0.0, 0.0, 0.0],
        }
    }
    pub fn set_stencil(&mut self, domain: &Domain) {
        let cell_size = self.settings.cell_size;
        let cutoff = self.settings.cutoff;
        let skin = self.settings.skin;

        let domain_min = [domain.xlo(), domain.ylo(), domain.zlo()];
        let domain_max = [domain.xhi(), domain.yhi(), domain.zhi()];

        let furthest_reach_min = [
            domain_min[0] - cutoff - skin,
            domain_min[1] - cutoff - skin,
            domain_min[2] - cutoff - skin,
        ];
        let furthest_reach_max = [
            domain_max[0] + cutoff + skin,
            domain_max[1] + cutoff + skin,
            domain_max[2] + cutoff + skin,
        ];

        let ncells_from_origin_min = [
            (furthest_reach_min[0] / cell_size).floor() as i64,
            (furthest_reach_min[1] / cell_size).floor() as i64,
            (furthest_reach_min[2] / cell_size).floor() as i64,
        ];
        let ncells_from_origin_max = [
            (furthest_reach_max[0] / cell_size).ceil() as i64,
            (furthest_reach_max[1] / cell_size).ceil() as i64,
            (furthest_reach_max[2] / cell_size).ceil() as i64,
        ];

        self.cell_origin = [
            ncells_from_origin_min[0] as f64 * cell_size,
            ncells_from_origin_min[1] as f64 * cell_size,
            ncells_from_origin_min[2] as f64 * cell_size,
        ];
        self.cells_per_axis = [
            (ncells_from_origin_max[0] - ncells_from_origin_min[0]) as usize,
            (ncells_from_origin_max[1] - ncells_from_origin_min[1]) as usize,
            (ncells_from_origin_max[2] - ncells_from_origin_min[2]) as usize,
        ];

        let max_ncells = (cutoff / cell_size).ceil() as i64;
        let cutoff_squared = cutoff * cutoff;
        for i in 1..=max_ncells {
            self.stencil.push([i, 0, 0]);
        }
        for i in -max_ncells..=max_ncells {
            for j in 1..=max_ncells {
                if ((i - 1) * (i - 1) + (j - 1) * (j - 1)) as f64 <= cutoff_squared {
                    self.stencil.push([i, j, 0]);
                }
            }
        }
        for i in -max_ncells..=max_ncells {
            for j in -max_ncells..=max_ncells {
                for k in 1..=max_ncells {
                    if ((i - 1) * (i - 1) + (j - 1) * (j - 1) + (k - 1) * (k - 1)) as f64
                        <= cutoff_squared
                    {
                        self.stencil.push([i, j, k]);
                    }
                }
            }
        }
        self.stencil.sort_by(|a, b| {
            let a_squared = a[0] * a[0] + a[1] * a[1] + a[2] * a[2];
            let b_squared = b[0] * b[0] + b[1] * b[1] + b[2] * b[2];
            a_squared.partial_cmp(&b_squared).unwrap()
        });
    }
    pub fn generate(&mut self, atoms: &[Atom]) {
        self.reset_cells();
        self.reset_neighbors(atoms.len());

        // Populate cells and neighbors
        for atom in atoms {
            let cell_index = self.get_3d_cell_index(atom.position);
            let cell_index_1d = self.index_to_1d_cell_index(cell_index);
            self.cells[cell_index_1d].push(atom.id);

            for stencil_index in &self.stencil {
                let neighbor_cell_index = [
                    (cell_index[0] as i64 + stencil_index[0]) as usize,
                    (cell_index[1] as i64 + stencil_index[1]) as usize,
                    (cell_index[2] as i64 + stencil_index[2]) as usize,
                ];
                if neighbor_cell_index[0] >= self.cells_per_axis[0]
                    || neighbor_cell_index[1] >= self.cells_per_axis[1]
                    || neighbor_cell_index[2] >= self.cells_per_axis[2]
                {
                    continue;
                }

                for neighbor_atom_id in
                    self.cells[self.index_to_1d_cell_index(neighbor_cell_index)].iter()
                {
                    // Skip if the neighbor is in the same cell and its neighbor list is not empty
                    if neighbor_cell_index == cell_index
                        && self.neighbors[*neighbor_atom_id].len() > 0
                    {
                        continue;
                    }
                    // Add neighbor to the atom's neighbor list
                    if *neighbor_atom_id != atom.id {
                        self.neighbors[atom.id].push(*neighbor_atom_id);
                    }
                }
            }
        }
    }
    fn reset_cells(&mut self) {
        let n_cells = self.cells_per_axis[0] * self.cells_per_axis[1] * self.cells_per_axis[2];
        if self.cells.len() != n_cells {
            self.cells.resize(n_cells, vec![]);
        }
        for i in 0..self.cells.len() {
            self.cells[i].clear();
        }
    }
    fn reset_neighbors(&mut self, n_atoms: usize) {
        if self.neighbors.len() != n_atoms {
            self.neighbors.resize(n_atoms, vec![]);
        }
        for i in 0..self.neighbors.len() {
            self.neighbors[i].clear();
        }
    }

    fn index_to_1d_cell_index(&self, cell_index: [usize; 3]) -> usize {
        cell_index[0] * self.cells_per_axis[1] * self.cells_per_axis[2]
            + cell_index[1] * self.cells_per_axis[2]
            + cell_index[2]
    }
    fn get_3d_cell_index(&self, position: [f64; 3]) -> [usize; 3] {
        [
            ((position[0] - self.cell_origin[0]) / self.settings.cell_size) as usize,
            ((position[1] - self.cell_origin[1]) / self.settings.cell_size) as usize,
            ((position[2] - self.cell_origin[2]) / self.settings.cell_size) as usize,
        ]
    }
    fn get_cell_index(&self, position: [f64; 3]) -> usize {
        let cell_index = self.get_3d_cell_index(position);
        self.index_to_1d_cell_index(cell_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Atom, Domain};

    #[test]
    fn test_neighbor_list() {
        let mut neighbor_list = NeighborList::new(NeighborListSettings::new(0.3, 2.5, 1, 0));
        neighbor_list.set_stencil(&Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0));
        let atoms = vec![Atom {
            id: 1,
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            force: [0.0, 0.0, 0.0],
            mass: 1.0,
            atom_type: 1,
        }];
        neighbor_list.generate(&atoms);
        assert_eq!(neighbor_list.neighbors[0], vec![]);
    }
}
