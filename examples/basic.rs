use jmd::atom_type::Basic;
use jmd::atomic::{LJCut, LJCutCoeff};
use jmd::*;

fn run(mut simulation: Simulation<atom_type::Basic, LJCut>) {
    let lj = LJCut::new(2.5);
    let coeff = LJCutCoeff::new(1.0, 1.0, 2.5);
    simulation.set_atomic_potential(lj);

    let lattice = Cubic::from_density(0.8);
    let rect = Rect::from_lattice(&lattice, [10, 10, 10]);
    let container = Container::from_rect_periodic(rect.clone());
    simulation.set_container(container);

    simulation.set_atom_types(vec![Basic::new(1.0)]);
    simulation
        .mut_atomic_potential()
        .set_coeff::<Basic>(0, 0, &coeff);

    let coords = lattice.coords_within_region(&rect, &[0.0, 0.0, 0.0]);
    simulation.atoms.add_atoms(0, coords);

    simulation.atoms.set_temperature(3.0);

    simulation.neighbor_list.set_skin_distance(0.3);
    simulation.set_nl_update(10, 0, true);

    simulation.add_compute("AvgVsq", Compute::AvgVsq);
    simulation.add_compute("Temperature", Compute::Temperature);
    simulation.add_compute("KineticE", Compute::KineticE);
    simulation.add_compute("PotentialE", Compute::PotentialE);
    simulation.add_compute("TotalE", Compute::TotalE);

    simulation.set_output(
        50,
        vec![
            "step",
            "AvgVsq",
            "Temperature",
            "KineticE",
            "PotentialE",
            "TotalE",
        ],
    );

    println!("Start");

    simulation.run(250);
}

fn main() {
    let mut sim: Jmd<atom_type::Basic, LJCut> = Jmd::new();

    sim.run(1, run);
}
