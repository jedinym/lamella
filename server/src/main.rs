mod threadpool;
mod response;
mod reqhandle;

extern crate simple_logger;

use std::net::TcpListener;

use threadpool::TcpTask;
use threadpool::Threadpool;

use reqhandle::handle_client;

use simple_logger::SimpleLogger;


fn main() {
    SimpleLogger::new().env().init().unwrap();

    let n_workers = 4;
    let mut pool = Threadpool::new(n_workers);
    let addr = "0.0.0.0";
    let port = "8000";

    let listener = TcpListener::bind(format!("{}:{}", addr, port)).unwrap();
    println!("Server listening at {}:{}", addr, port);
    println!("Workers running: {}", n_workers);

    loop {
        let (stream, _addr) = listener.accept().unwrap();
        let task = TcpTask::new(handle_client, stream);

        pool.add_task(task);
    }
}
