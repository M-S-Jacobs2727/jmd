use jmd::prelude::*;
use jmd::integrator::NVE;
use jmd::pairwise::{LJCut, LJCutCoeff};
use jmd::region::Rect;

#[test]
fn test_two_atoms_attraction() {
    let mut sim = SimulationBuilder::new()
        .with_box(Rect::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
        .with_pbc(
            BoundaryCondition::Periodic,
            BoundaryCondition::Periodic,
            BoundaryCondition::Periodic,
        )
        .with_neighbor_list(NeighborListSettings::new(0.3, 2.5, 1, 0))
        .with_integrator(NVE::new(0.005))
        .build();

    // Create two atoms with initial positions and velocities
    let coordinates = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    sim.add_atoms_at_coordinates(coordinates, 1.0, 1);

    // Add Lennard-Jones potential
    let mut ljcut = LJCut::new(2.5); // cutoff at 2.5 units
    let coeff = LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5); // epsilon=1.0, sigma=1.0
    ljcut.add_coeff(coeff);

    sim.add_pairwise(ljcut);

    // Initial distance between atoms
    let initial_distance = {
        let dx = sim.atoms().positions[1][0] - sim.atoms().positions[0][0];
        let dy = sim.atoms().positions[1][1] - sim.atoms().positions[0][1];
        let dz = sim.atoms().positions[1][2] - sim.atoms().positions[0][2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    // Run simulation
    sim.run(100);

    // Final distance between atoms
    let final_distance = {
        let dx = sim.atoms().positions[1][0] - sim.atoms().positions[0][0];
        let dy = sim.atoms().positions[1][1] - sim.atoms().positions[0][1];
        let dz = sim.atoms().positions[1][2] - sim.atoms().positions[0][2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    assert!(
        final_distance < initial_distance,
        "Atoms should have moved closer together, but distance increased from {} to {}",
        initial_distance,
        final_distance
    );
}

