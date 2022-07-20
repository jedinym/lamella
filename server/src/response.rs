pub struct Header {
    pub name: String,
    pub value: String,
}

pub struct ResponseBuilder {
    _status_code: u16,
    headers: Vec<Header>,
    body: String,
}

impl ResponseBuilder {
    pub fn new() -> ResponseBuilder {
        ResponseBuilder { _status_code: 200, headers: Vec::new(), body: String::new() }
    }

    pub fn status_code(mut self, code: u16) -> ResponseBuilder {
        self._status_code = code;
        self
    }

    pub fn add_header(mut self, header: Header) -> ResponseBuilder {
        self.headers.push(header);
        self
    }

    pub fn body(mut self, bytes: String) -> ResponseBuilder {
        self.body = bytes;
        self.headers.push(Header { name: "Content-Length".to_owned(), value: self.body.len().to_string() });
        self
    }

    pub fn success() -> ResponseBuilder {
        let a = ResponseBuilder::new();
        a
    }

    pub fn build(&mut self) -> Vec<u8> {
        let mut response = String::new();
        response.push_str("HTTP/1.1 ");
        response.push_str(&self._status_code.to_string());
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
