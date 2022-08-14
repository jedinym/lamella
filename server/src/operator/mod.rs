mod test;
mod route;

use std::sync::{Arc, Mutex};

use httparse::Request;
use test::TestOperator;

use self::route::{Route, RouteError};

pub type LockOperator = Arc<Mutex<Operator>>;


pub struct Operator {
    operators: Vec<Box<dyn Route>>,
}

impl Operator {
    pub fn new() -> Operator {
        let mut operators: Vec<Box<dyn Route + 'static>> = Vec::new();

        operators.push(Box::new(TestOperator::new()));

        return Operator { operators }
    }

    pub fn dispatch(&mut self, req: &Request) -> Result<String, RouteError> {
        for op in self.operators.iter_mut() {
            match op.resolve(req) {
                Ok(bytes) => return Ok(bytes),
                Err(RouteError::RouteNotMatched) => println!("not matched"),
                Err(RouteError::MissingParameter(msg)) => {
                    return Err(RouteError::MissingParameter(msg))
                },
                Err(_) => ()
            }
        }

        Err(RouteError::UnknownRoute)
    }
}
