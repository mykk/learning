use std::{sync::{mpsc, Arc, Mutex}, thread};
pub enum PoolCreationError {
    InvalidThreadCount
}

struct Worker {
    id: usize, 
    handle: Option<thread::JoinHandle<()>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move|| {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();        
                    },
                    Err(_) => { 
                        println!("Worker {id} disconnected; shutting down.");
                        break;    
                    }
                }
            }
        });

        Worker{id, handle : Some(handle)}
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            };
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::InvalidThreadCount);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size).map(|id| {
            Worker::new(id, Arc::clone(&receiver))
        }).collect();

        Ok(ThreadPool { workers, sender: Some(sender) })
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);

        if let Some(sender) = self.sender.as_ref() {
            sender.send(job).unwrap();
        }
    }
}
