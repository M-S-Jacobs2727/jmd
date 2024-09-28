pub fn distance_squared(coord1: &[f64; 3], coord2: &[f64; 3]) -> f64 {
    let x = coord1[0] - coord2[0];
    let y = coord1[1] - coord2[1];
    let z = coord1[2] - coord2[2];
    x * x + y * y + z * z
}
