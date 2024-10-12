use jmd::{self, parallel::worker::Worker, Jmd};

fn run(worker: &mut Worker) {
    let comm = jmd::parallel::communication::Communicator<jmd::AtomInfo>::new();

    let box_ = jmd::box_::Box_::new(
        0.0,
        10.0,
        0.0,
        10.0,
        0.0,
        10.0,
        jmd::box_::PBC::PP,
        jmd::box_::PBC::PP,
        jmd::box_::PBC::PP,
    );
    comm.init(&box_, worker.thread_ids());


    let mut atoms = jmd::Atoms::new();
    let rect = jmd::region::Rect::from_box(&box_);
    rect.add_random_atoms(&mut atoms, 10, 1, 1.0);
    let mut lj = jmd::ljcut::LJCut::new();
    lj.add_coeff(1, 1, 1.0, 1.0, 2.5).unwrap();
    let mut simulation = Simulation::new(comm);
    let mut verlet = jmd::verlet::Verlet::new(0.005, atoms);
    verlet.run(250);
}

fn main() -> Result<(), jmd::Error> {
    let sim = Jmd::new();

    println!("Hello, world!");
    sim.run(2, run)
}
