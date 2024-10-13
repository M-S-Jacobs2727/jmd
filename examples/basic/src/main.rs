use jmd::{self, Integrator};
fn run(worker: &mut jmd::worker::Worker) {
    let mut simulation = jmd::Simulation::new();
    simulation.init(worker);

    let rect = jmd::region::Rect::new(0.0, 10.0, 0.0, 10.0, 0.0, 10.0);
    let container = jmd::Container::from_rect(rect.clone());
    simulation.set_container(container);

    simulation.atoms.add_random_atoms(&rect.into(), 10, 1, 1.0);

    let mut lj = jmd::LJCut::new();
    lj.add_coeff(1, 1, 1.0, 1.0, 2.5).unwrap();
    simulation.set_atomic_potential(lj.into());
    simulation.set_neighbor_settings(jmd::UpdateSettings::new(10, 0, true));

    let mut verlet = jmd::Verlet::new();
    verlet.timestep = 0.005;
    verlet.run(&mut simulation, 250);
}

fn main() -> Result<(), jmd::Error> {
    let mut sim = jmd::Jmd::new();

    println!("Hello, world!");
    sim.run(2, run)
}
