mod response;
mod reqhandle;

extern crate simple_logger;
extern crate ctrlc;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, TryRecvError, Sender};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use std::thread::spawn;

use log::{info, error};
use threadpool::{Threadpool, Execute};

use reqhandle::handle_client;

use simple_logger::SimpleLogger;

fn set_ctrlc_handler(should_exit: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        info!("Attempting graceful shutdown");
        should_exit.store(true, Ordering::Relaxed);
    }).expect("Couldn't set interrupt handler");
}

fn listener_func(listener: TcpListener, sender: Sender<TcpStream>) {
    loop {
        let (stream, _addr) = listener.accept().unwrap();

        sender.send(stream);
    }
}

struct TcpTask {
    stream: TcpStream,
    handler: fn(TcpStream) -> ()
}

impl Execute for TcpTask {
    fn execute(self) {
        (self.handler)(self.stream);
    }
}


fn main() {
    SimpleLogger::new().env().init().unwrap();

    let n_workers = 4;
    let mut pool = Threadpool::new(n_workers);
    let addr = "0.0.0.0";
    let port = "8000";

    let should_exit = Arc::new(AtomicBool::new(false));
    set_ctrlc_handler(should_exit.clone());

    let listener = TcpListener::bind(format!("{}:{}", addr, port))
        .expect("Could not create TCP listener");

    println!("Server listening at {}:{}", addr, port);
    println!("Workers running: {}", n_workers);

    let (sender, receiver) = channel();
    let _listener_handle = spawn(move || listener_func(listener, sender));

    loop {
        if should_exit.load(Ordering::Relaxed) {
            pool.exit().expect("Failed to shutdown gracefully");
            info!("Graceful shutdown successful");
            return;
        }

        match receiver.try_recv() {
            Ok(stream) => {
                let task = TcpTask { stream, handler: handle_client};

                match pool.add_task(task) {
                    Err(err) => error!("Error adding task to threadpool: {:?}", err),
                    Ok(_) => ()
                }
            },
            Err(TryRecvError::Empty) => (),
            Err(_) => error!("Error while sending a stream!"),
        };
    }
}
