use jmd_engine::{Domain, EngineBuilder, LJCut, LJCutCoeff, NeighborListSettings, VelocityVerlet};

#[test]
fn test_two_atoms_attraction() {
    let mut engine = EngineBuilder::new()
        .with_domain(Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
        .with_neighbor_list_settings(NeighborListSettings::new(0.3, 2.5, 1, 0))
        .build()
        .unwrap();

    // Create two atoms with initial positions and velocities
    let coordinates = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];

    // Add Lennard-Jones potential
    let mut ljcut = LJCut::new(2.5); // cutoff at 2.5 units
    let coeff = LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5); // epsilon=1.0, sigma=1.0
    ljcut.add_coeff(coeff);

    engine.add_atoms_at_coordinates(coordinates, 1.0, 1);
    engine.add_constraint(Box::new(ljcut));
    engine.add_constraint(Box::new(VelocityVerlet::new(0.001)));

    // Initial distance between atoms
    let initial_distance = {
        let dx = engine.atom(1).position[0] - engine.atom(0).position[0];
        let dy = engine.atom(1).position[1] - engine.atom(0).position[1];
        let dz = engine.atom(1).position[2] - engine.atom(0).position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    // Run simulation
    engine.run(100);

    // Final distance between atoms
    let final_distance = {
        let dx = engine.atom(1).position[0] - engine.atom(0).position[0];
        let dy = engine.atom(1).position[1] - engine.atom(0).position[1];
        let dz = engine.atom(1).position[2] - engine.atom(0).position[2];
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
    let mut engine = EngineBuilder::new()
        .with_domain(Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
        .with_neighbor_list_settings(NeighborListSettings::new(0.3, 2.5, 1, 0))
        .build()
        .unwrap();

    // Create two atoms with initial positions but no initial velocities
    let coordinates = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];

    // Add Lennard-Jones potential
    let mut ljcut = LJCut::new(2.5); // cutoff at 2.5 units
    let coeff = LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5); // epsilon=1.0, sigma=1.0
    ljcut.add_coeff(coeff);

    engine.add_atoms_at_coordinates(coordinates, 1.0, 1);
    engine.add_constraint(Box::new(ljcut));
    engine.add_constraint(Box::new(VelocityVerlet::new(0.001)));

    // Initial distance should be exactly 2.0
    let initial_distance = {
        let dx = engine.atom(1).position[0] - engine.atom(0).position[0];
        let dy = engine.atom(1).position[1] - engine.atom(0).position[1];
        let dz = engine.atom(1).position[2] - engine.atom(0).position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };
    assert!(
        (initial_distance - 2.0).abs() < 1e-10,
        "Initial distance should be 2.0"
    );

    // Run simulation for longer since particles start at rest
    engine.run(1000);

    // Final distance between atoms
    let final_distance = {
        let dx = engine.atom(1).position[0] - engine.atom(0).position[0];
        let dy = engine.atom(1).position[1] - engine.atom(0).position[1];
        let dz = engine.atom(1).position[2] - engine.atom(0).position[2];
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
