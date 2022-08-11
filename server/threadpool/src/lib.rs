use std::sync::{Mutex, Arc};
use std::thread::{JoinHandle, spawn};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::panic::{catch_unwind, UnwindSafe};

pub trait Execute: Send + UnwindSafe + 'static {
    type Item: Send;

    fn execute(self) -> Self::Item;
}

enum WorkerMessage<T: Execute> {
    NewJob(T),
    Exit,
}

struct Worker<T: Execute> {
    receive_queue: Arc<Mutex<Receiver<WorkerMessage<T>>>>,
    sender: Sender<T::Item>
}

impl<T: Execute> Worker<T> {
    pub fn new(receive_queue: Arc<Mutex<Receiver<WorkerMessage<T>>>>, sender: Sender<T::Item>) -> JoinHandle<()> {
        let mut worker = Worker { receive_queue: receive_queue.clone() , sender };
        spawn(move || {
            worker.worker_function()
        })
    }

    fn worker_function(&mut self) {
        loop {
            let queue = self.receive_queue.lock().unwrap();
            let msg_result = queue.recv();

            match msg_result {
                Ok(WorkerMessage::NewJob(job)) => {
                    self.sender.send(job.execute());
                    //catch_unwind(move || job.execute()) TODO:
                },
                _ => return
            };
        }
    }
}

#[derive (Debug)]
pub enum ThreadPoolError {
    AddTaskError,
    ExitError,
}

pub struct Threadpool<T: Execute> {
    n_threads: usize,
    handles: Vec<JoinHandle<()>>,
    worker_queue_sender: Sender<WorkerMessage<T>>,
    worker_result_receiver: Receiver<T::Item>
}

impl<T: Execute> Threadpool<T> {
    pub fn new(n_threads: usize) -> Threadpool<T> {
        assert!(n_threads <= 16);

        let mut handles = Vec::with_capacity(n_threads);
        let (sender, receiver) = channel();
        let locked_rcvr = Arc::new(Mutex::new(receiver));

        let (result_sender, result_receiver) = channel();

        for _ in 0..n_threads {
            let handle = Worker::new(locked_rcvr.clone(), result_sender.clone());
            handles.push(handle);
        }

        Threadpool { n_threads, handles, worker_queue_sender: sender , worker_result_receiver: result_receiver }
    }

    pub fn add_task(&self, job: T) -> Result<(), ThreadPoolError> {
        let msg = WorkerMessage::NewJob(job);
        match self.worker_queue_sender.send(msg) {
            Ok(_) => Ok(()),
            Err(_) => Err(ThreadPoolError::AddTaskError),
        }
    }

    pub fn get_result(&self) -> Option<T::Item> {
        match self.worker_result_receiver.try_recv() {
            Ok(result) => Some(result),
            Err(_) => None,
        }
    }

    pub fn get_result_blocking(&self) -> T::Item {
        self.worker_result_receiver.recv().expect("Couldn't receive result")
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
