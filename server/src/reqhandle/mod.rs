use std::io::{Read, Write};
use std::net::TcpStream;

use crate::operator::LockOperator;
use crate::response::{Response, ResponseBuilder};

extern crate httparse;
use httparse::{Header, Request, EMPTY_HEADER};

use log::{error, info};

fn handle_request(req: &Request, operator: LockOperator) -> Response {
    let body = operator.lock().unwrap().dispatch(req).unwrap();
    let response = ResponseBuilder::success().body(body).build();

    response
}

fn parse_request<'a>(
    stream: &'a mut TcpStream,
    headers: &'a mut [Header<'a>],
    buf: &'a mut Vec<u8>,
) -> Result<Request<'a, 'a>, httparse::Error> {
    let mut request = Request::new(headers);

    stream.read(buf).unwrap(); // TODO: is this secret? is this safe?
    let _res = request.parse(buf)?;

    Ok(request)
}

fn send_response(stream: &mut TcpStream, response_bytes: Vec<u8>) {
    stream.write(&response_bytes).unwrap();
    stream.flush().unwrap();
}

fn log_request(req: &Request, response: &Response) {
    let method = &req.method.unwrap();
    let path = &req.path.unwrap();
    info! {"{} - {} - {}", method, path, response.status_code};
}

pub fn handle_client(mut stream: TcpStream, operator: LockOperator) {
    let mut buf = vec![0; 1024];
    let mut headers = [EMPTY_HEADER; 64];
    let request_result = parse_request(&mut stream, &mut headers, &mut buf);

    let response = match request_result {
        Err(errmsg) => {
            error!("Could not parse request: {}", errmsg);
            let response_bytes = ResponseBuilder::bad_request()
                .body(errmsg.to_string())
                .build()
                .bytes();
            response_bytes
        }
        Ok(request) => {
            println!("{:?}", request);
            let response = handle_request(&request, operator);
            log_request(&request, &response);
            let response_bytes = response.bytes();
            response_bytes
        }
    };

    send_response(&mut stream, response)
}
