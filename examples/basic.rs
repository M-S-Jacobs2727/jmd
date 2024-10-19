use jmd::{self, AtomicPotentialTrait, IntegratorTrait, UpdateSettings};
fn run(simulation: &mut jmd::Simulation) {
    let rect = jmd::region::Rect::new(0.0, 10.0, 0.0, 10.0, 0.0, 10.0);
    let container = jmd::Container::from_rect(rect.clone());
    simulation.set_container(container);

    simulation.atoms.add_random_atoms(&rect.into(), 10, 0, 1.0);
    dbg!(&simulation.atoms.positions);
    let mut lj = jmd::LJCut::new(1, 2.5);
    let force_distance = lj.cutoff_distance();
    lj.set_coeff(0.into(), 0.into(), 1.0, 1.0, 2.5).unwrap();
    simulation.set_atomic_potential(lj.into());

    let mut nl = jmd::NeighborList::new(simulation.container(), 1.75, force_distance, 0.3);
    nl.set_update_settings(UpdateSettings::new(10, 0, true));
    simulation.set_neighbor_list(nl);

    let mut verlet = jmd::Verlet::new();
    verlet.timestep = 0.005;
    println!("Start");

    verlet.run(simulation, 250);
}

fn main() -> Result<(), jmd::Error> {
    let mut sim = jmd::Jmd::new();

    sim.run(1, run)
}
