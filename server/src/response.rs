
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Clone for Header {
    fn clone(&self) -> Header {
        Header {name: self.name.clone(), value: self.value.clone()}
    }
}

pub struct Response {
    pub status_code: u16,
    pub headers: Vec<Header>,
    pub body: String
}


impl Clone for Response {
    fn clone(&self) -> Response {
        Response {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body: self.body.clone()
        }
    }
}

pub struct ResponseBuilder {
    response: Response
}


impl Response {
    pub fn bytes(&self) -> Vec<u8> {
        let mut response = String::new();
        response.push_str("HTTP/1.1 ");
        response.push_str(&self.status_code.to_string());
        response.push_str(" Testing-phrase ");
        response.push_str("\r\n");

        for header in &self.headers {
            let a: String = format!("{}:{} ", header.name, header.value);
            response.push_str(&a);
        }

        response.push_str("\r\n");
        response.push_str(&self.body);

        response.as_bytes().to_vec()
    }
}

impl ResponseBuilder {
    pub fn new() -> ResponseBuilder {
        let response = Response { status_code: 200, headers: Vec::new(), body: String::new() };
        ResponseBuilder { response }
    }

    pub fn status_code(mut self, code: u16) -> ResponseBuilder {
        self.response.status_code = code;
        self
    }

    pub fn add_header(mut self, header: Header) -> ResponseBuilder {
        self.response.headers.push(header);
        self
    }

    pub fn body(mut self, bytes: String) -> ResponseBuilder {
        self.response.body = bytes;
        self
    }

    pub fn success() -> ResponseBuilder {
        let a = ResponseBuilder::new();
        a
    }

    pub fn bad_request() -> ResponseBuilder {
        ResponseBuilder::new()
            .status_code(400)
    }

    pub fn build(&mut self) -> Response {
        self.response.clone()
    }
}
