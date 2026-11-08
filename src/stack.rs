pub trait Stack {
    fn new(size: usize) -> Self;
    fn top(&mut self) -> *mut u8;
}

#[derive(Debug)]
pub struct VecStack {
    mem: Vec<u128>
}

impl Stack for VecStack {
    fn new(size: usize) -> Self {
        let s = size.div_ceil(16);

        Self {
            mem: vec![0; s]
        }
    }
    fn top(&mut self) -> *mut u8 {
        unsafe {
            self.mem.as_mut_ptr().add(self.mem.len()) as *mut u8
        }
    }
}