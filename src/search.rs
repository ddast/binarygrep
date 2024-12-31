use crate::buffer::Buffer;

pub trait Search {
    fn new(pat: Vec<u8>) -> Self;
    fn search(&self, data: &Buffer, offset: usize) -> Option<usize>;
}
