use std::sync::{Arc, Mutex};


pub type LockOperator = Arc<Mutex<Operator>>;


pub struct Operator {
    test: i32,
}

impl Operator {
    pub fn new() -> Operator {
        Operator { test: 0 }
    }
}
