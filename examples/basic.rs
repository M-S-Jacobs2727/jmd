use jmd::{
    self, atomic::AtomicPotential, compute::Compute, Integrator, Lattice, OutputSpec,
    UpdateSettings,
};
fn run(simulation: &mut jmd::Simulation) -> Result<(), jmd::Error> {
    let lattice = jmd::Cubic::from_density(0.8);
    let rect = jmd::Rect::from_lattice(&lattice, [10, 10, 10]);

    let container = jmd::Container::from_rect(rect.clone());
    simulation.set_container(container);

    let coords = lattice.coords_within_region(&rect, &[0.0, 0.0, 0.0]);
    simulation.atoms.add_atoms(0, 1.0, coords);
    dbg!(simulation.atoms.num_atoms());

    simulation.atoms.set_temperature(3.0)?;

    let mut lj = jmd::atomic::LJCut::new(1, 2.5);
    let force_distance = lj.cutoff_distance();
    lj.set_coeff(0.into(), 0.into(), 1.0, 1.0, 2.5)?;
    simulation.set_atomic_potential(lj);

    let mut nl = jmd::NeighborList::new(simulation.container(), force_distance, 0.3);
    // dbg!(&nl);
    nl.set_update_settings(UpdateSettings::new(10, 0, true));
    simulation.set_neighbor_list(nl);

    let output = jmd::Output {
        every: 50,
        values: vec![
            OutputSpec::Step,
            OutputSpec::Compute(Compute::Temperature),
            OutputSpec::Compute(Compute::KineticE),
            OutputSpec::Compute(Compute::PotentialE),
            OutputSpec::Compute(Compute::TotalE),
        ],
    };
    simulation.set_output(output);

    let mut verlet = jmd::Verlet::new();
    verlet.timestep = 0.005;
    println!("Start");

    verlet.run(simulation, 250);
    Ok(())
}

fn main() -> Result<(), jmd::Error> {
    let mut sim = jmd::Jmd::new();

    sim.run(1, run)
}
