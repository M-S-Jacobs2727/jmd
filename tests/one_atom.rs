use jmd_engine::{Atom, Engine};

#[test]
fn test_single_atom_velocity() {
    let mut engine = Engine::new();
    engine.dt = 0.001; // Set small time step

    // Create atom with initial position at origin and velocity in x direction
    let atom = Atom {
        position: [0.0, 0.0, 0.0],
        velocity: [1.0, 0.0, 0.0],
        force: [0.0, 0.0, 0.0],
        mass: 1.0,
        atom_type: 1,
        id: 1,
    };

    engine.add_atom(atom);

    // Run for 10 steps
    engine.run(10);

    // Check final position
    // With dt = 0.001 and velocity = 1.0, after 10 steps
    // position should be approximately 0.01 in x direction
    let expected_x = 0.01;
    assert!((engine.atoms[0].position[0] - expected_x).abs() < 1e-10);
    assert!(engine.atoms[0].position[1].abs() < 1e-10);
    assert!(engine.atoms[0].position[2].abs() < 1e-10);

    // Velocity should remain constant as there are no forces
    assert!((engine.atoms[0].velocity[0] - 1.0).abs() < 1e-10);
    assert!(engine.atoms[0].velocity[1].abs() < 1e-10);
    assert!(engine.atoms[0].velocity[2].abs() < 1e-10);
}
