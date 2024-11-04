use jmd::atom_type::Basic;
use jmd::atomic::{LJCut, LJCutCoeff};
use jmd::*;

fn run(mut sim: Simulation<Basic, LJCut>) {
    let lattice = Cubic::from_density(0.8);
    let rect = Rect::from_lattice(&lattice, [10, 10, 10]);
    let container = Container::from_rect_periodic(rect.clone());
    let coords = lattice.coords_within_region(&rect, &[0.0, 0.0, 0.0]);

    sim.set_atom_types(vec![Basic::new(1.0)]);

    sim.set_atomic_potential(LJCut::new(2.5));
    sim.set_atomic_coeff(0, 0, &LJCutCoeff::new(1.0, 1.0, 2.5));

    sim.set_container(container);

    sim.add_atoms(0, coords);

    sim.set_temperature(3.0);

    sim.set_nl_skin_distance(0.3);
    sim.set_nl_update(10, 0, true);

    sim.add_compute("AvgVsq", Compute::AvgVsq);
    sim.add_compute("Temperature", Compute::Temperature);
    sim.add_compute("KineticE", Compute::KineticE);
    sim.add_compute("PotentialE", Compute::PotentialE);
    sim.add_compute("TotalE", Compute::TotalE);

    sim.set_output(
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

    sim.run(250);
}

fn main() {
    let mut app: Jmd<Basic, LJCut> = Jmd::new();

    app.run(1, run);
}
