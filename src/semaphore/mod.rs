use std::sync::{Condvar, Mutex};

pub struct Semaphore {
    count: Mutex<usize>,
    condvar: Condvar,
}

const IS_VERBOSE: bool = false;

impl Semaphore {
    pub fn new(count: usize) -> Semaphore {
        Semaphore {
            count: Mutex::new(count),
            condvar: Condvar::new(),
        }
    }

    // pub fn set_verbose(&self, verbose: bool) {
    //     let mut self_verbose = self.verbose.lock().unwrap();
    //     *self_verbose = verbose;
    // }

    pub fn wait(&self) {
        let mut count = self.count.lock().unwrap();
        // let verbose = self.verbose.lock().unwrap();

        while *count == 0 {
            count = self.condvar.wait(count).unwrap();
        }
        *count -= 1;
        if IS_VERBOSE {
            println!("wait {}", count);
        }
    }

    pub fn signal(&self) {
        let mut count = self.count.lock().unwrap();
        // let verbose = self.verbose.lock().unwrap();

        *count += 1;
        if IS_VERBOSE {
            println!("signal {}", count);
        }
        self.condvar.notify_one();
    }
}
