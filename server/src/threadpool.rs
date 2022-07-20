use std::fmt::Debug;
use std::sync::{Mutex, Arc, Condvar};
use std::collections::VecDeque;
use std::thread::{JoinHandle, spawn};

#[derive(Debug)]
pub struct ConcurrentQueue <T> {
    queue_mtx: Mutex<VecDeque<T>>,
    cvar: Arc<(Mutex<bool>, Condvar)>
}

type Task = dyn FnOnce() -> () + Send;

impl<T> ConcurrentQueue<T> {
    pub fn new() -> ConcurrentQueue<T> {
        let cvar = Arc::new((Mutex::new(false), Condvar::new()));
        ConcurrentQueue { queue_mtx: Mutex::new(VecDeque::new()), cvar}
    }

    pub fn pop(&self) -> Option<T> {
        let mut queue = self.queue_mtx.lock().unwrap();

        queue.pop_front()
    }

    pub fn append(&self, val: T) {
        let mut queue = self.queue_mtx.lock().unwrap();
        queue.push_back(val);

        // notify threads that new values are available
        let (lock, cvar) = &*self.cvar;
        let mut ready = lock.lock().unwrap();
        *ready = true;
        cvar.notify_one();
    }

    pub fn wait_pop(&self) -> T {
        let (cvar_lock, cvar) = &*self.cvar;
        let mut ready = cvar_lock.lock().unwrap();

        // wait until queue has items to pop
        while ! *ready {
            ready = cvar.wait(ready).unwrap();
        }

        let val = self.pop().unwrap();

        if self.queue_mtx.lock().unwrap().is_empty() {
            *ready = false;
        }

        val
    }
}

fn worker_function(task_queue: Arc<ConcurrentQueue<Box<Task>>>)
{
    loop {
        let task = task_queue.wait_pop();
        task();
    }
}

pub struct Threadpool
{
    pub task_queue: Arc<ConcurrentQueue<Box<Task>>>,
    workers: Vec<JoinHandle<()>>,
}

impl Threadpool
{
    pub fn new(n: u8) -> Threadpool {
        let task_queue: Arc<ConcurrentQueue<Box<Task>>> = Arc::new(ConcurrentQueue::new());
        let mut workers = Vec::new();

        for _ in 0..n {
            let queue_clone_ptr = task_queue.clone();
            let handle = spawn(|| worker_function(queue_clone_ptr));
            workers.push(handle);
        }
        return Threadpool { task_queue,  workers }
    }
}
