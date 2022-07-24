use std::net::TcpListener;

mod threadpool;
use threadpool::TcpTask;
use threadpool::Threadpool;

mod response;
mod reqhandle;
use reqhandle::{handle_client};

extern crate simple_logger;
extern crate log;

use simple_logger::SimpleLogger;
use log::{info, warn, error};


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
