pub mod linear;
pub mod scope;

#[derive(Clone, Copy, Debug)]
pub enum AllocError {
    OutOfMemory,
}

