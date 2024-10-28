use jmd::atom_type::Basic;
use jmd::atomic::LJCut;
use jmd::*;

fn run(worker: &Worker<Basic>) {
    let atoms: Atoms<Basic> = Atoms::new();
    let lj = LJCut::new(2.5);

    let lattice = Cubic::from_density(0.8);
    let rect = Rect::from_lattice(&lattice, [10, 10, 10]);
    let container = Container::from_rect(rect.clone());

    let mut simulation: Simulation<Basic, LJCut> = Simulation::new(atoms, lj, container);
    simulation.connect(Box::new(worker));

    simulation.set_atom_types(vec![Basic::new(1.0)]);

    let coords = lattice.coords_within_region(&rect, &[0.0, 0.0, 0.0]);
    simulation.atoms.add_atoms(0, coords);

    simulation.atoms.set_temperature(3.0);

    simulation
        .mut_atomic_potential()
        .set_coeff::<Basic>(0, 0, 1.0, 1.0, 2.5);

    simulation.set_neighbor_list(0.3, UpdateSettings::new(10, 0, true));

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

    let mut verlet = Verlet::new();
    verlet.timestep = 0.005;
    println!("Start");

    verlet.run(&mut simulation, 250);
}

fn main() {
    let mut sim: Jmd<atom_type::Basic> = Jmd::new();

    sim.run(1, run);
}
