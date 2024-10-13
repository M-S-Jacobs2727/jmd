// TODO: add more errortypes

/// Error types
#[derive(Debug)]
pub enum Error {
    AtomicPotentialError,
    IntegratorError,
    NeighborListError,
    OtherError,
}
