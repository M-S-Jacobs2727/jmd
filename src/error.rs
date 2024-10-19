// TODO: add more errortypes

/// Error types
#[derive(Clone, Copy, Debug)]
pub enum Error {
    AtomicPotentialError,
    IntegratorError,
    NeighborListError,
    OtherError,
}
