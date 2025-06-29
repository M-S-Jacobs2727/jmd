use jmd_engine::{
    BoundaryCondition, Domain, Engine, LJCut, LJCutCoeff, NeighborListSettings, SimulationBox,
    SystemBuilder, VelocityVerlet,
};

#[test]
fn test_two_atoms_attraction() {
    let mut system = SystemBuilder::new()
        .with_simulation_box(SimulationBox::new(
            Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0),
            [
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
            ],
        ))
        .with_neighbor_list_settings(NeighborListSettings::new(0.3, 2.5, 1, 0))
        .build();

    // Create two atoms with initial positions and velocities
    let coordinates = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    system.add_atoms_at_coordinates(coordinates, 1.0, 1);
    system.atoms[0].velocity = [1.0, 0.0, 0.0];
    system.atoms[1].velocity = [-1.0, 0.0, 0.0];

    // Create engine
    let mut engine = Engine::new();

    // Add Lennard-Jones potential
    let mut ljcut = LJCut::new(2.5); // cutoff at 2.5 units
    let coeff = LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5); // epsilon=1.0, sigma=1.0
    ljcut.add_coeff(coeff);

    engine.add_constraint(Box::new(ljcut));
    engine.add_constraint(Box::new(VelocityVerlet::new(0.001)));

    // Initial distance between atoms
    let initial_distance = {
        let dx = system.atom(1).position[0] - system.atom(0).position[0];
        let dy = system.atom(1).position[1] - system.atom(0).position[1];
        let dz = system.atom(1).position[2] - system.atom(0).position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    // Run simulation
    engine.run(100, &mut system);

    // Final distance between atoms
    let final_distance = {
        let dx = system.atom(1).position[0] - system.atom(0).position[0];
        let dy = system.atom(1).position[1] - system.atom(0).position[1];
        let dz = system.atom(1).position[2] - system.atom(0).position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    assert!(
        final_distance < initial_distance,
        "Atoms should have moved closer together, but distance increased from {} to {}",
        initial_distance,
        final_distance
    );
}

#[test]
fn test_two_atoms_drift() {
    let mut system = SystemBuilder::new()
        .with_simulation_box(SimulationBox::new(
            Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0),
            [
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
            ],
        ))
        .with_neighbor_list_settings(NeighborListSettings::new(0.3, 2.5, 1, 0))
        .build();

    // Create two atoms with initial positions but no initial velocities
    let coordinates = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];

    // Add Lennard-Jones potential
    let mut ljcut = LJCut::new(2.5); // cutoff at 2.5 units
    let coeff = LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5); // epsilon=1.0, sigma=1.0
    ljcut.add_coeff(coeff);

    system.add_atoms_at_coordinates(coordinates, 1.0, 1);
    system.atoms[0].velocity = [0.0, 0.0, 0.0];
    system.atoms[1].velocity = [0.0, 0.0, 0.0];

    // Create engine
    let mut engine = Engine::new();

    engine.add_constraint(Box::new(ljcut));
    engine.add_constraint(Box::new(VelocityVerlet::new(0.001)));

    // Initial distance should be exactly 2.0
    let initial_distance = {
        let dx = system.atom(1).position[0] - system.atom(0).position[0];
        let dy = system.atom(1).position[1] - system.atom(0).position[1];
        let dz = system.atom(1).position[2] - system.atom(0).position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };
    assert!(
        (initial_distance - 2.0).abs() < 1e-10,
        "Initial distance should be 2.0"
    );

    // Run simulation for longer since particles start at rest
    engine.run(1000, &mut system);

    // Final distance between atoms
    let final_distance = {
        let dx = system.atom(1).position[0] - system.atom(0).position[0];
        let dy = system.atom(1).position[1] - system.atom(0).position[1];
        let dz = system.atom(1).position[2] - system.atom(0).position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    // Check that atoms have moved closer together
    assert!(
        final_distance < initial_distance,
        "Atoms should have moved closer together, but distance changed from {} to {}",
        initial_distance,
        final_distance
    );
}
