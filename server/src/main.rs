use std::io::Write;
use std::io::Read;
use std::net::{TcpStream, TcpListener};

pub mod threadpool;
use threadpool::TcpTask;
use threadpool::Threadpool;

pub mod response;
use response::ResponseBuilder;

extern crate httparse;
extern crate simple_logger;
extern crate log;

use httparse::{EMPTY_HEADER, Request, Header};
use simple_logger::SimpleLogger;
use log::{info, warn, error};


fn handle_request(_req: Request) -> Vec<u8> {
    let test = response::ResponseBuilder::new()
        .status_code(200)
        .add_header(response::Header {name: "hello".to_owned(), value: "boi".to_owned()})
        .add_header(response::Header {name: "hi!".to_owned(), value: "tom!".to_owned()})
        .body("BODYYY!".to_owned()).build();

    test
}

fn parse_request<'a>(
    stream: &'a mut TcpStream,
    headers: &'a mut [Header<'a>],
    buf: &'a mut Vec<u8>)
    -> Result<Request<'a, 'a>, httparse::Error> {
    let mut request = Request::new(headers);

    stream.read(buf).unwrap();  // TODO: is this secret? is this safe?
    let _res = request.parse(buf)?;

    Ok(request)
}


fn send_response(stream: &mut TcpStream, response_bytes: Vec<u8>) {
    stream.write(&response_bytes).unwrap();
    stream.flush().unwrap();
}


fn handle_client(stream: &mut TcpStream) {
    let mut buf = vec![0; 1024];
    let mut headers = [EMPTY_HEADER; 64];
    let request_result = parse_request(stream, &mut headers, &mut buf);

    let response = match request_result {
        Err(errmsg) => {
            error!("Could not parse request: {}", errmsg);
            let response_bytes = ResponseBuilder::bad_request()
                .body(errmsg.to_string())
                .build();
            response_bytes
        }
        Ok(request) => {
            info!("{:?}", request);
            let response_bytes = handle_request(request);
            response_bytes
        }
    };

    send_response(stream, response)
}


fn main() {
    SimpleLogger::new().env().init().unwrap();

    let mut pool = Threadpool::new(4);

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    loop {
        let (stream, _addr) = listener.accept().unwrap();
        let task = TcpTask::new(handle_client, stream);

        pool.add_task(task);
    }
}
