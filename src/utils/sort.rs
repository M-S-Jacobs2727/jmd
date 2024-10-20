use std::mem::swap;

/// Create the indices that can be used to sort the given array.
///
/// ```rust
/// sorted_vec[sort_indices[i]] = unsorted_vec[i]
/// ```
pub fn get_sort_indices(bin_indices: &Vec<usize>) -> Vec<usize> {
    if bin_indices.is_empty() {
        return Vec::new();
    }
    let len = bin_indices.len();
    let mut counts: Vec<usize> = Vec::new();
    counts.resize(*bin_indices.iter().max().unwrap(), 0);

    for b in bin_indices {
        counts[*b] += 1;
    }
    for i in 1..counts.len() {
        counts[i] += counts[i - 1];
    }

    let mut indices: Vec<usize> = Vec::new();
    indices.resize(len, 0);
    for (i, b) in bin_indices.iter().enumerate().rev() {
        let j = *b;
        counts[j] -= 1;
        let k = counts[j];
        indices[i] = k;
    }
    indices
}
/// Sorts a vector of atom properties using a vector of indices.
///
/// ```rust
/// sorted_vec[sort_indices[i]] = unsorted_vec[i]
/// ```
pub fn sort_atoms<T: Clone + Copy>(sort_indices: &Vec<usize>, unsorted_vec: &mut Vec<T>, dummy: T) {
    assert!(sort_indices.len() == unsorted_vec.len());
    let mut output: Vec<T> = Vec::new();
    output.resize(unsorted_vec.len(), dummy);
    for (i, &idx) in sort_indices.iter().enumerate() {
        output[idx] = unsorted_vec[i];
    }
    swap(&mut output, unsorted_vec);
}
