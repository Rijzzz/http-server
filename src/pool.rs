use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

struct Worker {
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

impl ThreadPool {
    /// # Panics
    /// Panics if size is zero
    #[must_use]
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(tx) = &self.sender {
            let _ = tx.send(job);
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = spawn(move || {
            loop {
                let res = receiver.lock().ok().and_then(|rx| rx.recv().ok());
                match res {
                    Some(job) => job(),
                    None => break,
                }
            }
        });

        Self {
            _id: id,
            thread: Some(thread),
        }
    }
}
