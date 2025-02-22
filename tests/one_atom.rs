use jmd_engine::{Domain, EngineBuilder, NeighborListSettings, VelocityVerlet};

use float_cmp::assert_approx_eq;

#[test]
fn test_single_atom_velocity() {
    let mut engine = EngineBuilder::new()
        .with_domain(Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
        .with_neighbor_list_settings(NeighborListSettings::new(0.3, 2.5, 1, 0))
        .build()
        .unwrap();
    engine.add_atoms_at_coordinates(vec![[0.0, 0.0, 0.0]], 1.0, 1);
    engine.add_constraint(Box::new(VelocityVerlet::new(0.001)));

    // Run for 10 steps
    engine.run(10);

    // Check final position
    // With dt = 0.001 and velocity = 1.0, after 10 steps
    // position should be approximately 0.01 in x direction
    let expected_x = 0.01;
    assert_approx_eq!(f64, engine.atom(0).position[0], expected_x, epsilon = 1e-10);
    assert_approx_eq!(f64, engine.atom(0).position[1], 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, engine.atom(0).position[2], 0.0, epsilon = 1e-10);

    // Velocity should remain constant as there are no forces
    assert_approx_eq!(f64, engine.atom(0).velocity[0], 1.0, epsilon = 1e-10);
    assert_approx_eq!(f64, engine.atom(0).velocity[1], 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, engine.atom(0).velocity[2], 0.0, epsilon = 1e-10);
}
