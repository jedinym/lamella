use std::net::TcpStream;
use std::sync::{Mutex, Arc, Condvar};
use std::collections::VecDeque;
use std::thread::{JoinHandle, spawn};

pub struct ConcurrentQueue <T> {
    queue_mtx: Mutex<VecDeque<T>>,
    cvar: Arc<(Mutex<bool>, Condvar)>
}

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

pub trait Execute: Send + 'static {
    fn execute(&mut self) -> ();
}

pub struct TcpTask {
    handler: fn(&mut TcpStream) -> (),
    stream: TcpStream
}

impl TcpTask {
    pub fn new(handler: fn(&mut TcpStream), stream: TcpStream) -> TcpTask {
        TcpTask { handler, stream }
    }
}

impl Execute for TcpTask {
    fn execute(&mut self) {
        (self.handler)(&mut self.stream);
    }
}


pub struct Threadpool<T: Execute>
{
    task_queue: Arc<ConcurrentQueue<T>>,
}

impl<T> Threadpool<T>
where T: Execute
{
    pub fn new(n: u8) -> Threadpool<T> {
        let task_queue: Arc<ConcurrentQueue<T>> = Arc::new(ConcurrentQueue::new());

        for _ in 0..n {
            Self::make_thread(task_queue.clone());
        }

        return Threadpool { task_queue }
    }

    fn make_thread(task_queue: Arc<ConcurrentQueue<T>>) -> JoinHandle<()> {
            let queue_clone_ptr = task_queue.clone();
            spawn(move || Self::worker_function(queue_clone_ptr))
    }

    fn worker_function(task_queue: Arc<ConcurrentQueue<T>>)
    {
        loop {
            let mut task = task_queue.wait_pop();
            task.execute();
        }
    }

    pub fn add_task(&mut self, task: T) {
        self.task_queue.append(task);
    }
}
