use std::panic::{catch_unwind, UnwindSafe};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

pub trait Execute: Send + UnwindSafe + 'static {
    fn execute(self);
}

enum WorkerMessage<T: Execute> {
    NewJob(T),
    Exit,
}

struct Worker<T: Execute> {
    receive_queue: Arc<Mutex<Receiver<WorkerMessage<T>>>>,
}

impl<T: Execute> Worker<T> {
    pub fn new(receive_queue: Arc<Mutex<Receiver<WorkerMessage<T>>>>) -> JoinHandle<()> {
        let mut worker = Worker {
            receive_queue: receive_queue.clone(),
        };
        spawn(move || worker.worker_function())
    }

    fn worker_function(&mut self) {
        loop {
            let queue = self.receive_queue.lock().unwrap();
            let msg_result = queue.recv();

            match msg_result {
                Ok(WorkerMessage::NewJob(job)) => {
                    catch_unwind(move || job.execute());
                }
                _ => return,
            };
        }
    }
}

#[derive(Debug)]
pub enum ThreadPoolError {
    AddTaskError,
    ExitError,
}

pub struct Threadpool<T: Execute> {
    n_threads: usize,
    handles: Vec<JoinHandle<()>>,
    worker_queue_sender: Sender<WorkerMessage<T>>,
}

impl<T: Execute> Threadpool<T> {
    pub fn new(n_threads: usize) -> Threadpool<T> {
        assert!(n_threads <= 16);

        let mut handles = Vec::with_capacity(n_threads);
        let (sender, receiver) = channel();
        let locked_rcvr = Arc::new(Mutex::new(receiver));

        for _ in 0..n_threads {
            let handle = Worker::new(locked_rcvr.clone());
            handles.push(handle);
        }

        Threadpool {
            n_threads,
            handles,
            worker_queue_sender: sender,
        }
    }

    pub fn add_task(&self, job: T) -> Result<(), ThreadPoolError> {
        let msg = WorkerMessage::NewJob(job);
        match self.worker_queue_sender.send(msg) {
            Ok(_) => Ok(()),
            Err(_) => Err(ThreadPoolError::AddTaskError),
        }
    }

    pub fn exit(&mut self) -> Result<(), ThreadPoolError> {
        for _ in 0..self.n_threads {
            match self.worker_queue_sender.send(WorkerMessage::Exit) {
                Ok(_) => (),
                Err(_) => return Err(ThreadPoolError::AddTaskError),
            }
        }

        while !self.handles.is_empty() {
            self.handles.remove(0).join().expect("Couldn't join thread");
        }

        Ok(())
    }
}
