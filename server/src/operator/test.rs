use super::route::{Route, RouteError};
use std::{fs::File, io::Write};


pub struct TestOperator {
    filename: String,
}


impl TestOperator {
    pub fn new() -> TestOperator {
        TestOperator { filename: "test".to_owned() }
    }

    pub fn mkfile(&self) -> () {
        let mut file = File::create(&self.filename).unwrap();
        file.write_all(b"operator operating!").unwrap();
    }
}


impl Route for TestOperator {
    fn resolve(&mut self, req: &httparse::Request) -> Result<String, RouteError> {
        if req.path.unwrap() == "/test" {
            self.mkfile();
        }

        Ok("Hello".to_owned())
    }
}

