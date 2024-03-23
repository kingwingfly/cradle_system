//! Local cradle, running on local machine, does not require network signal.

use std::{
    cell::RefCell,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

/// type alias for `Result<T, Box<dyn std::error::Error + Send>>`
pub type BoxResult<T> = Result<T, Box<dyn std::error::Error + Send>>;

/// A baby that cries after a certain time.
pub struct Baby {
    time: usize,
    cry: Box<dyn Fn() -> BoxResult<()> + Send>,
}

impl Baby {
    /// Instantiates a new baby.
    pub fn new<F>(time: usize, cry: F) -> Self
    where
        F: Fn() -> BoxResult<()> + Send + 'static,
    {
        Self {
            time,
            cry: Box::new(cry),
        }
    }

    fn cry(&self) -> BoxResult<()> {
        (self.cry)()?;
        Ok(())
    }
}

/// A cradle that holds babies.
pub struct Cradle {
    babies: Arc<Mutex<RefCell<Vec<Baby>>>>,
    tx: Sender<Signal>,
    jh: thread::JoinHandle<BoxResult<()>>,
}

impl Cradle {
    /// Instantiates a new cradle.
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let babies: Arc<Mutex<RefCell<Vec<Baby>>>> = Arc::new(Mutex::new(RefCell::new(Vec::new())));
        let babies_c = babies.clone();
        let jh = thread::spawn(move || {
            if Signal::Start == rx.recv().unwrap() {
                let mut elapsed = 0;
                loop {
                    let signal = rx.try_recv();
                    match signal {
                        Ok(signal) => match signal {
                            Signal::Reset => elapsed = 0,
                            Signal::Cry => {
                                let babies = babies_c.lock().unwrap();
                                for baby in babies.borrow().iter() {
                                    baby.cry()?;
                                }
                            }
                            Signal::Stop => break,
                            _ => {}
                        },
                        _ => {
                            let babies = babies_c.lock().unwrap();
                            for baby in babies.borrow().iter() {
                                if elapsed >= baby.time {
                                    baby.cry()?;
                                }
                            }
                            thread::sleep(Duration::from_secs(1));
                            elapsed += 1;
                        }
                    }
                }
            }
            Ok(())
        });
        Self { babies, tx, jh }
    }

    /// Pushes a baby into the cradle.
    pub fn put_baby(&mut self, baby: Baby) {
        let mut_babies = self.babies.lock().unwrap();
        mut_babies.borrow_mut().push(baby);
    }

    /// Starts the cradle.
    pub fn start(&self) {
        self.tx.send(Signal::Start).unwrap();
    }

    /// Resets the cradle's elapsed time, so that babies will not cry.
    pub fn reset(&self) {
        self.tx.send(Signal::Reset).unwrap();
    }

    /// Gracefully stops the cradle.
    pub fn stop(&self) {
        self.tx.send(Signal::Stop).unwrap();
    }

    /// Forces the babies to cry.
    pub fn cry(&self) {
        self.tx.send(Signal::Cry).unwrap();
    }

    /// Joins the cradle thread.
    pub fn join(self) -> thread::Result<BoxResult<()>> {
        self.jh.join()
    }
}

impl Default for Cradle {
    fn default() -> Self {
        Cradle::new()
    }
}

#[derive(PartialEq)]
enum Signal {
    Reset,
    Cry,
    Start,
    Stop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cradle() {
        let mut cradle = Cradle::new();
        cradle.put_baby(Baby::new(2, || {
            println!("Baby 1: Waaaaaah!");
            Ok(())
        }));
        cradle.put_baby(Baby::new(3, || {
            println!("Baby 2: Waaaaaah!");
            Ok(())
        }));
        cradle.start();
        cradle.put_baby(Baby::new(1, || {
            println!("Baby 3: Waaaaaah!");
            Ok(())
        }));
        thread::sleep(Duration::from_secs(7));
        cradle.reset();
        thread::sleep(Duration::from_secs(1));
        cradle.reset();
        thread::sleep(Duration::from_secs(1));
        cradle.cry();
        cradle.stop();
        cradle.join().unwrap().unwrap();
    }
}
