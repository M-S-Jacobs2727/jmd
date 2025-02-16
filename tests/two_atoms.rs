use jmd_engine::{Atom, Engine, LJCut, LJCutCoeff};

#[test]
fn test_two_atoms_attraction() {
    let mut engine = Engine::new();
    engine.dt = 0.001; // Small timestep for stability

    // Create two atoms with initial positions and velocities
    let atom1 = Atom {
        position: [-1.0, 0.0, 0.0],
        velocity: [0.1, 0.0, 0.0],
        force: [0.0, 0.0, 0.0],
        mass: 1.0,
        atom_type: 1,
        id: 1,
    };

    let atom2 = Atom {
        position: [1.0, 0.0, 0.0],
        velocity: [-0.1, 0.0, 0.0],
        force: [0.0, 0.0, 0.0],
        mass: 1.0,
        atom_type: 1,
        id: 2,
    };

    // Add Lennard-Jones potential
    let mut ljcut = LJCut::new(2.5); // cutoff at 2.5 units
    let coeff = LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5); // epsilon=1.0, sigma=1.0
    ljcut.add_coeff(coeff);

    engine.add_atom(atom1);
    engine.add_atom(atom2);
    engine.add_force(Box::new(ljcut));

    // Initial distance between atoms
    let initial_distance = {
        let dx = engine.atoms[1].position[0] - engine.atoms[0].position[0];
        let dy = engine.atoms[1].position[1] - engine.atoms[0].position[1];
        let dz = engine.atoms[1].position[2] - engine.atoms[0].position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    // Run simulation
    engine.run(100);

    // Final distance between atoms
    let final_distance = {
        let dx = engine.atoms[1].position[0] - engine.atoms[0].position[0];
        let dy = engine.atoms[1].position[1] - engine.atoms[0].position[1];
        let dz = engine.atoms[1].position[2] - engine.atoms[0].position[2];
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
    let mut engine = Engine::new();
    engine.dt = 0.001; // Small timestep for stability

    // Create two atoms with initial positions but no initial velocities
    let atom1 = Atom {
        position: [-1.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0], // No initial velocity
        force: [0.0, 0.0, 0.0],
        mass: 1.0,
        atom_type: 1,
        id: 1,
    };

    let atom2 = Atom {
        position: [1.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0], // No initial velocity
        force: [0.0, 0.0, 0.0],
        mass: 1.0,
        atom_type: 1,
        id: 2,
    };

    // Add Lennard-Jones potential
    let mut ljcut = LJCut::new(2.5); // cutoff at 2.5 units
    let coeff = LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5); // epsilon=1.0, sigma=1.0
    ljcut.add_coeff(coeff);

    engine.add_atom(atom1);
    engine.add_atom(atom2);
    engine.add_force(Box::new(ljcut));

    // Initial distance should be exactly 2.0
    let initial_distance = {
        let dx = engine.atoms[1].position[0] - engine.atoms[0].position[0];
        let dy = engine.atoms[1].position[1] - engine.atoms[0].position[1];
        let dz = engine.atoms[1].position[2] - engine.atoms[0].position[2];
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
        let dx = engine.atoms[1].position[0] - engine.atoms[0].position[0];
        let dy = engine.atoms[1].position[1] - engine.atoms[0].position[1];
        let dz = engine.atoms[1].position[2] - engine.atoms[0].position[2];
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
