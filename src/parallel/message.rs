pub enum Message {
    Float3(Vec<[f64; 3]>),
    Float(Vec<f64>),
    Int3(Vec<[i32; 3]>),
    Types(Vec<u32>),
    Idxs(Vec<usize>),
}
