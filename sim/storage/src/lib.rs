pub mod linear;
pub mod scope_stack;

#[derive(Clone, Copy, Debug)]
pub enum AllocError {
    OutOfMemory,
}

