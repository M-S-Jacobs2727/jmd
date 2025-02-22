use crate::{atom::Atom, Domain};

pub struct NeighborListSettings {
    pub cell_size: f64,
    pub cutoff: f64,
    pub update_every: usize,
    pub delay_update: usize,
}
impl NeighborListSettings {
    pub fn new(cell_size: f64, cutoff: f64, update_every: usize, delay_update: usize) -> Self {
        Self {
            cell_size,
            cutoff,
            update_every,
            delay_update,
        }
    }
    pub fn neighbor_list(self, domain: Domain) -> NeighborList {
        let mut neighbor_list = NeighborList::new(self);
        neighbor_list.set_stencil(domain);
        neighbor_list
    }
}

pub struct NeighborList {
    pub cells: Vec<Vec<usize>>,
    pub neighbors: Vec<Vec<usize>>,
    pub settings: NeighborListSettings,
    pub cells_per_axis: [usize; 3],
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
        }
    }
    pub fn set_stencil(&mut self, domain: Domain) {
        let cell_size = self.settings.cell_size;
        let cutoff = self.settings.cutoff;
        self.cells_per_axis = [
            (domain.lx() / cell_size).ceil() as usize,
            (domain.ly() / cell_size).ceil() as usize,
            (domain.lz() / cell_size).ceil() as usize,
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
    }
    pub fn generate(&mut self, atoms: &[Atom]) {
        self.cells.clear();
        self.neighbors.clear();

        self.cells.resize(
            self.cells_per_axis[0] * self.cells_per_axis[1] * self.cells_per_axis[2],
            vec![],
        );
        self.neighbors.resize(atoms.len(), vec![]);
        for atom in atoms {
            let cell_index = self.get_cell_index(atom.position);
            self.cells[cell_index].push(atom.id);
        }
        for atom in atoms {
            let cell_index = self.get_3d_cell_index(atom.position);
            for stencil in &self.stencil {
                let neighbor_cell_index = [
                    cell_index[0] + stencil[0] as usize,
                    cell_index[1] + stencil[1] as usize,
                    cell_index[2] + stencil[2] as usize,
                ];
                for neighbor_atom_id in
                    self.cells[self.index_to_1d_cell_index(neighbor_cell_index)].iter()
                {
                    if *neighbor_atom_id != atom.id {
                        self.neighbors[atom.id].push(*neighbor_atom_id);
                    }
                }
            }
        }
    }
    fn index_to_1d_cell_index(&self, cell_index: [usize; 3]) -> usize {
        cell_index[0] * self.cells_per_axis[1] * self.cells_per_axis[2]
            + cell_index[1] * self.cells_per_axis[2]
            + cell_index[2]
    }
    fn get_3d_cell_index(&self, position: [f64; 3]) -> [usize; 3] {
        [
            (position[0] / self.settings.cell_size) as usize,
            (position[1] / self.settings.cell_size) as usize,
            (position[2] / self.settings.cell_size) as usize,
        ]
    }
    fn get_cell_index(&self, position: [f64; 3]) -> usize {
        let cell_index = self.get_3d_cell_index(position);
        cell_index[0] * self.cells_per_axis[1] * self.cells_per_axis[2]
            + cell_index[1] * self.cells_per_axis[2]
            + cell_index[2]
    }
}
