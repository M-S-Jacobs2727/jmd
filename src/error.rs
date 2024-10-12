// TODO: add more errortypes

#[derive(Debug)]
pub enum Error {
    AtomicPotentialError,
    IntegratorError,
    NeighborListError,
    OtherError,
}
