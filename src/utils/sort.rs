use std::mem::swap;

/// Create the indices that can be used to sort the given array.
///
/// ```rust
/// use jmd::utils::get_sort_indices;
///
/// let unsorted_vec = vec![2, 0, 1];
/// let indices = get_sort_indices(&unsorted_vec);
/// assert_eq!(indices, vec![1, 2, 0]);
///
/// assert_eq!(0, unsorted_vec[indices[0]]);
/// assert_eq!(1, unsorted_vec[indices[1]]);
/// assert_eq!(2, unsorted_vec[indices[2]]);
/// ```
pub fn get_sort_indices(input_vec: &Vec<usize>) -> Vec<usize> {
    let max_value = input_vec.iter().max();
    let new_len = match max_value {
        Some(v) => *v + 1,
        None => return Vec::new(),
    };
    let mut counts: Vec<usize> = Vec::new();
    counts.resize(new_len, 0);

    for b in input_vec {
        counts[*b] += 1;
    }
    for i in 1..new_len {
        counts[i] += counts[i - 1];
    }

    let len = input_vec.len();
    let mut output: Vec<usize> = Vec::new();
    output.resize(len, 0);
    for i_inv in 0..len {
        let i = len - 1 - i_inv;
        let j = input_vec[i];
        counts[j] -= 1;
        output[counts[j]] = i;
    }
    output
}
/// Sorts a vector of atom properties using a vector of indices.
///
/// ```rust
/// use jmd::utils::{get_sort_indices, sort_atoms};
///
/// let mut sort_keys = vec![2usize, 0, 1];
/// let indices = get_sort_indices(&sort_keys);
///
/// let mut other_prop = vec![1.0, 2.0, 3.0];
/// let mut other_prop2 = vec![[1.0, 1.0, 1.0], [3.0, 3.0, 3.0], [2.0, 2.0, 2.0]];
///
/// sort_atoms(&indices, &mut sort_keys);
/// sort_atoms(&indices, &mut other_prop);
/// sort_atoms(&indices, &mut other_prop2);
///
/// assert_eq!(sort_keys, vec![0usize, 1, 2]);
/// assert_eq!(other_prop, vec![2.0, 3.0, 1.0]);
/// assert_eq!(other_prop2, vec![[3.0, 3.0, 3.0], [2.0, 2.0, 2.0], [1.0, 1.0, 1.0]]);
/// ```
pub fn sort_atoms<T: Clone + Copy>(sort_indices: &Vec<usize>, unsorted_vec: &mut Vec<T>) {
    assert!(sort_indices.len() == unsorted_vec.len());
    let mut output: Vec<T> = sort_indices.iter().map(|&idx| unsorted_vec[idx]).collect();
    swap(&mut output, unsorted_vec);
}
