use ndarray::{arr0, arr1, Array, Array1, Array2, Axis, Dimension, RemoveAxis};

use crate::{
    neighbor,
    region::{Region, RegionTrait},
    utils,
};

/// Atom properties during simulation, not including forces
pub struct Atoms {
    pub ids: Array1<usize>,
    pub types: Array1<u32>,
    pub positions: Array2<f64>,
    pub velocities: Array2<f64>,
    pub masses: Array1<f64>,
    pub nlocal: usize,
}
impl Atoms {
    pub fn new() -> Self {
        Atoms {
            ids: ndarray::arr1(&[]),
            types: ndarray::arr1(&[]),
            positions: ndarray::arr2::<f64, 3>(&[]),
            velocities: ndarray::arr2::<f64, 3>(&[]),
            masses: ndarray::arr1(&[]),
            nlocal: 0,
        }
    }
    pub fn num_atoms(&self) -> usize {
        self.ids.len()
    }
    pub fn id_to_idx(&self, id: usize) -> Option<usize> {
        self.ids.iter().position(|x| *x == id)
    }
    pub fn sort_atoms_by_bin(&mut self, bins: &neighbor::Grid) -> Array1<usize> {
        let bin_indices = bins.coords_to_linear_indices(&self.positions);
        let sort_indices = utils::get_sort_indices(&bin_indices);

        utils::sort_atoms(&sort_indices, &mut self.ids);
        utils::sort_atoms(&sort_indices, &mut self.types);
        utils::sort_atoms(&sort_indices, &mut self.positions);
        utils::sort_atoms(&sort_indices, &mut self.velocities);
        utils::sort_atoms(&sort_indices, &mut self.masses);

        bins.coords_to_linear_indices(&self.positions)
    }
    pub fn add_random_atoms(
        &mut self,
        region: &Region,
        num_atoms: usize,
        atom_type: u32,
        mass: f64,
    ) {
        let atom_id = match self.ids.iter().max() {
            Some(j) => j + 1,
            None => 0,
        };
        self.ids.reserve(Axis(0), num_atoms);
        self.types.reserve(Axis(0), num_atoms);
        self.positions.reserve(Axis(0), num_atoms);
        self.velocities.reserve(Axis(0), num_atoms);
        self.masses.reserve(Axis(0), num_atoms);
        self.nlocal += num_atoms;

        for i in 0..num_atoms {
            self.ids.push(Axis(0), arr0(atom_id + i).view());
            self.types.push(Axis(0), arr0(atom_type).view());
            self.masses.push(Axis(0), arr0(mass).view());
            self.velocities.push(Axis(0), arr1(&[0.0, 0.0, 0.0]).view());
            self.positions
                .push(Axis(0), arr1(&region.get_random_coord()).view());
        }
    }
    pub fn remove_idxs(&mut self, atom_idxs: Array1<usize>) {
        let num_local = atom_idxs.iter().filter(|&i| *i < self.nlocal).count();
        self.nlocal -= num_local;
        fn filter_by_idx<T: Copy + num_traits::identities::Zero, D: Dimension + RemoveAxis>(
            atom_idxs: &Array1<usize>,
            vec: &Array<T, D>,
        ) -> Array<T, D> {
            let mut shape = vec.shape();
            shape[0] -= atom_idxs.len();
            let mut out = Array::zeros(shape);

            atom_idxs.iter().enumerate().for_each(|(i, idx)| {
                out.view_mut()
                    .index_axis_mut(Axis(0), i)
                    .assign(&vec.index_axis(Axis(0), *idx));
            });

            out
        }

        self.ids = filter_by_idx(&atom_idxs, &self.ids);
        self.types = filter_by_idx(&atom_idxs, &self.types);
        self.masses = filter_by_idx(&atom_idxs, &self.masses);
        self.positions = filter_by_idx(&atom_idxs, &self.positions);
        self.velocities = filter_by_idx(&atom_idxs, &self.velocities);
    }
}
