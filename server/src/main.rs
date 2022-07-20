use std::io::Write;
use std::io::Read;
use std::net::{TcpStream, TcpListener};

pub mod threadpool;
use threadpool::Threadpool;

pub mod response;

extern crate httparse;
use httparse::{EMPTY_HEADER, Request, Header};

fn get_stream_bytes(stream: &mut TcpStream) -> Vec<u8> {
    // TODO: this needs to be better
    let mut buf = vec![0; 1024];
    stream.read(&mut buf).unwrap();

    buf
}

fn parse_request<'a>(bytes: &'a Vec<u8>, headers: &'a mut [Header<'a>]) -> Request<'a, 'a> {
    let mut request = Request::new(headers);
    let _result = request.parse(&bytes).expect("Malformed http request");

    request
}

fn handle_request(req: Request) -> Vec<u8> {
    let test = response::ResponseBuilder::new()
        .status_code(200)
        .add_header(response::Header {name: "hello".to_owned(), value: "boi".to_owned()})
        .add_header(response::Header {name: "hi!".to_owned(), value: "tom!".to_owned()})
        .body("BODYYY!".to_owned()).build();

    test
}

fn handle_client(mut stream: TcpStream) {
    let buffer: Vec<u8> = get_stream_bytes(&mut stream);

    let mut headers = [EMPTY_HEADER; 64];
    let request = parse_request(&buffer, &mut headers);
    println!("{:?}", request);

    let response = handle_request(request);

    stream.write(&response).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let pool = Threadpool::new(4);

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    loop {
        let (stream, _addr) = listener.accept().unwrap();
        let task = Box::new(move || handle_client(stream));

        pool.task_queue.append(task);
    }
}
