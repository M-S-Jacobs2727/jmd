use jmd::prelude::*;
use jmd::integrator::NVE;
use jmd::region::Rect;

use float_cmp::assert_approx_eq;

#[test]
fn test_single_atom_velocity() {
    let mut sim = SimulationBuilder::new()
        .with_box(Rect::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
        .with_pbc(
            BoundaryCondition::Periodic,
            BoundaryCondition::Periodic,
            BoundaryCondition::Periodic,
        )
        .with_neighbor_list(NeighborListSettings::new(0.3, 2.5, 1, 0))
        .with_integrator(NVE::new(0.001))
        .build();
    sim.add_atoms_at_coordinates(vec![[0.0, 0.0, 0.0]], 1.0, 1);
    sim.set_velocities(1.0, 0.0, 0.0);

    // Run for 10 steps
    sim.run(10);

    // Check final position
    // With dt = 0.001 and velocity = 1.0, after 10 steps
    // position should be approximately 0.01 in x direction
    let expected_x = 0.01;
    let atoms = sim.atoms();
    assert_approx_eq!(f64, atoms.positions[0][0], expected_x, epsilon = 1e-10);
    assert_approx_eq!(f64, atoms.positions[0][1], 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, atoms.positions[0][2], 0.0, epsilon = 1e-10);

    // Velocity should remain constant as there are no forces
    assert_approx_eq!(f64, atoms.velocities[0][0], 1.0, epsilon = 1e-10);
    assert_approx_eq!(f64, atoms.velocities[0][1], 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, atoms.velocities[0][2], 0.0, epsilon = 1e-10);
}
