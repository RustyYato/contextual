use std::cell::UnsafeCell;

pub struct Context<T> {
    blocks: UnsafeCell<Vec<*mut T>>,
}

impl<T> Context<T> {
    pub fn new() -> Self { loop {} }
}
