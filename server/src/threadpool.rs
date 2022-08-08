use std::net::TcpStream;
use std::sync::{Mutex, Arc};
use std::thread::{JoinHandle, spawn};
use std::sync::mpsc::{Sender, Receiver, channel, SendError};

pub struct Job {
    stream: TcpStream,
    func: fn(TcpStream)
}

impl Job {
    pub fn new(stream: TcpStream, func: fn(TcpStream)) -> Job {
        Job { stream, func }
    }
}

pub enum Message {
    NewJob(Job),
    Exit,
}

struct Worker {
    receive_queue: Arc<Mutex<Receiver<Message>>>,
}

impl Worker {
    pub fn new(receive_queue: Arc<Mutex<Receiver<Message>>>) -> JoinHandle<()> {
        let mut worker = Worker { receive_queue: receive_queue.clone() };
        spawn(move || {
            worker.worker_function()
        })
    }

    fn worker_function(&mut self) {
        loop {
            let queue = self.receive_queue.lock().unwrap();
            let msg = queue.recv().unwrap();
            match msg {
                Message::Exit => return,
                Message::NewJob(job) => (job.func)(job.stream)
            }
        }
    }
}

pub struct Threadpool {
    n_threads: usize,
    handles: Vec<JoinHandle<()>>,
    sender: Sender<Message>
}

impl Threadpool {
    pub fn new(n_threads: usize) -> Threadpool {
        assert!(n_threads <= 16);

        let mut handles = Vec::with_capacity(n_threads);
        let (sender, receiver) = channel();
        let locked_rcvr = Arc::new(Mutex::new(receiver));

        for _ in 0..n_threads {
            let handle = Worker::new(locked_rcvr.clone());
            handles.push(handle);
        }

        Threadpool { n_threads, handles, sender }
    }

    pub fn add_task(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        self.sender.send(msg)
    }

    pub fn exit(&mut self) -> Result<(), SendError<Message>> {
        for _ in 0..self.n_threads {
            self.add_task(Message::Exit)?;
        }

        while !self.handles.is_empty() {
            self.handles.remove(0).join().expect("Couldn't join thread");
        }

        Ok(())
    }
}
