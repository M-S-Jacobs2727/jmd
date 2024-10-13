use jmd::{self, parallel};
// TODO: inspect why syntax highlighting won't work
// TODO: determine best API
fn run(worker: &mut Worker) {
    let domain = parallel::Domain::new();

    let container = jmd::Container::new(
        0.0,
        10.0,
        0.0,
        10.0,
        0.0,
        10.0,
        jmd::PBC::PP,
        jmd::PBC::PP,
        jmd::PBC::PP,
    );
    domain.init(&container, worker.thread_ids());

    let mut atoms = jmd::Atoms::new();
    let rect = jmd::Rect::from_box(&container);
    rect.add_random_atoms(&mut atoms, 10, 1, 1.0);
    let mut lj = jmd::LJCut::new();
    lj.add_coeff(1, 1, 1.0, 1.0, 2.5).unwrap();
    let mut simulation = jmd::Simulation::new(domain);
    let mut verlet = jmd::Verlet::new(0.005, simulation);
    verlet.run(250);
}

fn main() -> Result<(), jmd::Error> {
    let sim = jmd::Jmd::new();

    println!("Hello, world!");
    sim.run(2, run)
}
