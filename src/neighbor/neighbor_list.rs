use crate::neighbor::Bins;

pub struct NeighborList {
    bins: Bins,
    force_distance: f64,
    cutoff_distance: f64,
    neighbors: Vec<Vec<usize>>,
}
impl NeighborList {
    pub fn new(num_atoms: usize, bins: Bins, force_distance: f64, cutoff_distance: f64) -> Self {
        let mut neighbors: Vec<Vec<usize>> = Vec::new();
        assert!(
            force_distance > 0.0,
            "Force cutoff distance must be positive"
        );
        assert!(
            cutoff_distance > force_distance,
            "Neighbor list cutoff distance must be greater than force cutoff distance"
        );
        neighbors.resize(num_atoms, Vec::new());
        Self {
            bins,
            force_distance,
            cutoff_distance,
            neighbors,
        }
    }
    pub fn neighbors(&self) -> &Vec<Vec<usize>> {
        &self.neighbors
    }
    pub fn force_distance(&self) -> f64 {
        self.force_distance
    }
    pub fn cutoff_distance(&self) -> f64 {
        self.cutoff_distance
    }
    pub fn bins(&self) -> &Bins {
        &self.bins
    }
    pub fn update(&mut self, positions: &Vec<[f64; 3]>, bin_numbers: &Vec<usize>) {
        let num_atoms = positions.len();
        assert!(
            num_atoms == bin_numbers.len() && num_atoms == self.neighbors.len(),
            "Number of atoms in positions vector and bin_numbers vector should be equal"
        );
        assert!(
            bin_numbers
                .iter()
                .take(num_atoms - 1)
                .zip(bin_numbers.iter().skip(1))
                .all(|(i, j)| i <= j),
            "Atoms should be sorted before updating neighbor list"
        );

        let mut bin_start_indices: Vec<Option<usize>> = Vec::new();
        for &bin_num in bin_numbers {
            if bin_start_indices.len() > bin_num {
                continue;
            }

            while bin_start_indices.len() < bin_num {
                bin_start_indices.push(None);
            }
            bin_start_indices.push(Some(bin_num))
        }

        // do stuff
        // for each bin, loop over closest bins in predetermined shape
        // (close to spherical, maybe start as cubic?)
        // read LAMMPS paper again
    }
}
