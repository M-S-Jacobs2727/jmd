use jmd_engine::{
    BoundaryCondition, Domain, Engine, NeighborListSettings, SimulationBox, SystemBuilder,
    VelocityVerlet,
};

use float_cmp::assert_approx_eq;

#[test]
fn test_single_atom_velocity() {
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
    system.add_atoms_at_coordinates(vec![[0.0, 0.0, 0.0]], 1.0, 1);
    system.atoms[0].velocity = [1.0, 0.0, 0.0];

    let mut engine = Engine::new();
    engine.add_constraint(Box::new(VelocityVerlet::new(0.001)));

    // Run for 10 steps
    engine.run(10, &mut system);

    // Check final position
    // With dt = 0.001 and velocity = 1.0, after 10 steps
    // position should be approximately 0.01 in x direction
    let expected_x = 0.01;
    let atom = system.atom(0);
    assert_approx_eq!(f64, atom.position[0], expected_x, epsilon = 1e-10);
    assert_approx_eq!(f64, atom.position[1], 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, atom.position[2], 0.0, epsilon = 1e-10);

    // Velocity should remain constant as there are no forces
    assert_approx_eq!(f64, atom.velocity[0], 1.0, epsilon = 1e-10);
    assert_approx_eq!(f64, atom.velocity[1], 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, atom.velocity[2], 0.0, epsilon = 1e-10);
}
