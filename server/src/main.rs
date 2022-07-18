use std::io;
use std::io::Read;
use std::net::{TcpStream, TcpListener};
use std::thread;

extern crate httparse;
use httparse::{EMPTY_HEADER, Request, Header};

fn get_stream_bytes(stream: &mut TcpStream) -> Vec<u8> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();

    buf
}

fn parse_request<'a>(bytes: &'a Vec<u8>, headers: &'a mut [Header<'a>]) -> Request<'a, 'a> {
    let mut request = Request::new(headers);
    let _result = request.parse(&bytes).expect("Malformed http request");

    request
}

fn handle_client(stream: &mut TcpStream) {
    let buffer: Vec<u8> = get_stream_bytes(stream);

    let mut headers = [EMPTY_HEADER; 64];
    let request = parse_request(&buffer, &mut headers);
    println!("{:?}", request);
}

fn main() -> Result<(), io::Error> {

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    loop {
        let (mut stream, _addr) = listener.accept().unwrap();
        thread::spawn(move || handle_client(&mut stream));
    }

    Ok(())
}
