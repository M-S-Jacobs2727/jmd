use jmd::{self, region::Region, AtomicPotential, Integrator, Simulation};
// TODO: inspect why syntax highlighting won't work
// TODO: determine best API
fn run(worker: &mut jmd::worker::Worker) {
    let mut simulation: Simulation<jmd::LJCut> = jmd::Simulation::new();
    simulation.init(worker);
    let container = jmd::Container::new(
        0.0,
        10.0,
        0.0,
        10.0,
        0.0,
        10.0,
        jmd::BC::PP,
        jmd::BC::PP,
        jmd::BC::PP,
    );
    simulation.set_container(container);

    let rect = jmd::region::Rect::from_box(simulation.container());
    rect.add_random_atoms(&mut simulation.atoms, 10, 1, 1.0);

    let mut lj = jmd::LJCut::new();
    lj.add_coeff(1, 1, 1.0, 1.0, 2.5).unwrap();
    simulation.set_atomic_potential(lj);

    let mut verlet = jmd::Verlet::new();
    verlet.timestep = 0.005;
    verlet.run(&mut simulation, 250);
}

fn main() -> Result<(), jmd::Error> {
    let mut sim = jmd::Jmd::new();

    println!("Hello, world!");
    sim.run(2, run)
}
