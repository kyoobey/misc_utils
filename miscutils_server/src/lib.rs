


#[macro_use]
extern crate log;

use std::{
	process::exit,                          // instantly exit the program
	sync::{
		Arc, Mutex,                         // to share the job queue across workers
		mpsc::{ channel, Sender, Receiver } // to manage the job queue
	},
	thread::{ JoinHandle, spawn }           // for creating threads
};



pub struct ThreadPool {
	workers: Vec<Worker>,
	sender: Option<Sender<Job>>
}

impl ThreadPool {



	/// create a new ThreadPool.
	/// 
	/// `size` is the number of workers in the pool
	/// 
	/// # panics
	/// 
	/// - the `new` function will panic if the size is zero
	/// 
	/// ```
	/// let pool = ThreadPool::new(4);
	/// ```
	/// 
	/// - if `size` is too large, the whole server will panic
	pub fn new (size: usize) -> Self {
		assert!(size > 0);

		// create the queue
		let (sender, receiver) = channel();
		let receiver = Arc::new(Mutex::new(receiver));

		// create the workers
		let mut workers = Vec::with_capacity(size);
		for i in 0..size {
			workers.push(Worker::new(i+1, Arc::clone(&receiver)));
		}
		info!("created {:?} workers", size);

		// create the threadpool
		Self { workers, sender: Some(sender) }
	}


	/// put a closure inside the threadpool queue
	/// which will be executed when there's a worker available
	/// 
	/// ```
	/// let pool = ThreadPool::new(3); // 3 threads
	/// for job_id in 0..8 {           // 8 jobs
	/// 	pool.execute(|| {
	/// 		for i in 0..5 {
	/// 			std::thread::sleep(0.5);
	/// 			println!("job {:?}: {:?}", job_id, i);
	/// 		}
	/// 	});
	/// }
	/// ```
	pub fn execute<F> (&self, closure: F)
	where
		F: FnOnce() + Send + 'static
	{
		let job = Box::new(closure);
		self.sender.as_ref().unwrap().send(job).unwrap();
	}



}

impl Drop for ThreadPool {

	/// when a `ThreadPool` is dropped, it's inner channel (queue) sender is also dropped
	/// signaling all the workers to finish their current job if any, then terminate
	fn drop (&mut self) {

		drop(self.sender.take());

		for worker in &mut self.workers {
			if let Some(thread) = worker.thread.take() {
				thread.join().unwrap_or_else(|e| {
					error!("{:?}", e);
					error!("fatal deadlock, shutting down worker {:?} failed, exiting\n", worker.id);
					exit(1);
				});
			}
		}

	}

}



type Job = Box<dyn FnOnce() + Send + 'static>;



struct Worker {
	id: usize,
	thread: Option<JoinHandle<()>>
}

impl Worker {

	fn new (
		id: usize,
		receiver: Arc<Mutex<Receiver<Job>>>
	) -> Self {

		let thread = spawn(move || loop {
			let message = {
				receiver
					.lock().unwrap_or_else(|_| {
						error!("the threadpool queue mutex has been poisoned, exiting fatally");
						exit(1);
					})
					.recv()
			};

			match message {
				Ok(job) => {
					debug!("worker {:?}: got a job", id);
					job();
				},
				Err(_) => {
					warn!("worker {:?}: disconnected, shutting down", id);
					break;
				}
			}
		});

		Self { id, thread: Some(thread) }

	}

}


