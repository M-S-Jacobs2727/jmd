use std::mem::swap;

use ndarray::{Array1, Axis, Dimension, RemoveAxis, Slice};

/// Create the indices that can be used to sort the given array.
///
/// ```rust
/// sorted_vec[sort_indices[i]] = unsorted_vec[i]
/// ```
pub fn get_sort_indices(bin_indices: &ndarray::Array1<usize>) -> ndarray::Array1<usize> {
    if bin_indices.is_empty() {
        return ndarray::arr1(&[]);
    }
    let len = bin_indices.len();
    let mut counts: Vec<usize> = Vec::new();
    counts.resize(bin_indices.iter().max().unwrap().clone(), 0);

    for b in bin_indices {
        counts[*b] += 1;
    }
    for i in 1..counts.len() {
        counts[i] += counts[i - 1];
    }

    let mut indices: Array1<usize> = Array1::zeros(len);
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
pub fn sort_atoms<T, D>(
    sort_indices: &ndarray::Array1<usize>,
    unsorted_vec: &mut ndarray::Array<T, D>,
) where
    T: Clone + Copy + num_traits::identities::Zero,
    D: Dimension + RemoveAxis + Copy,
{
    assert!(sort_indices.len() == unsorted_vec.len());
    let mut output: ndarray::Array<T, D> = ndarray::Array::zeros(unsorted_vec.dim());
    sort_indices.indexed_iter().for_each(|(i, idx)| {
        output
            .slice_each_axis_mut(|a| {
                if a.axis == Axis(0) {
                    Slice::from(*idx..=*idx)
                } else {
                    Slice::from(..)
                }
            })
            .assign(&unsorted_vec.slice_axis(Axis(0), Slice::from(i..=i)));
    });
    swap(&mut output, unsorted_vec);
}
