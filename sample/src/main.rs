use jmd;
use jmd::region::Region;

fn main() {
    let mut sim = jmd::Simulation::new(jmd::box_::Box_::new(
        0.0,
        10.0,
        0.0,
        10.0,
        0.0,
        10.0,
        jmd::box_::PBC::PP,
        jmd::box_::PBC::PP,
        jmd::box_::PBC::PP,
    ));
    let rect = jmd::region::Rect::from_box(&sim.box_);
    rect.add_random_atoms(&mut sim, 10, 1, 1.0);
    println!("Hello, world!");
}
