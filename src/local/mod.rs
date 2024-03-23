//! Local cradle, running on local machine, does not require network signal.

use std::{
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};

/// type alias for `Result<T, Box<dyn std::error::Error + Send>>`
pub type BoxResult<T> = Result<T, Box<dyn std::error::Error + Send>>;

/// A baby that cries after a certain time.
pub trait Baby {
    /// The cry behavior of the baby.
    ///
    /// # Arguments
    /// elapsed: The elapsed time in seconds.
    fn cry(&mut self, elapsed: usize) -> BoxResult<()>;
}

/// A cradle that holds babies.
pub struct Cradle {
    tx: Sender<Signal>,
    jh: thread::JoinHandle<BoxResult<()>>,
}

impl Cradle {
    /// Instantiates a new cradle.
    pub fn new<B>(mut babies: Vec<B>) -> Self
    where
        B: Baby + Send + Sync + 'static,
    {
        let (tx, rx) = channel();
        let jh = thread::spawn(move || {
            if Signal::Start == rx.recv().unwrap() {
                let mut elapsed = 0;
                loop {
                    let signal = rx.try_recv();
                    match signal {
                        Ok(signal) => match signal {
                            Signal::Reset => elapsed = 0,
                            Signal::Stop => break,
                            _ => {}
                        },
                        _ => {
                            for baby in babies.iter_mut() {
                                baby.cry(elapsed)?;
                            }
                            thread::sleep(Duration::from_secs(1));
                            elapsed += 1;
                        }
                    }
                }
            }
            Ok(())
        });
        Self { tx, jh }
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

    /// Joins the cradle thread.
    pub fn join(self) -> thread::Result<BoxResult<()>> {
        self.jh.join()
    }
}

#[derive(PartialEq)]
enum Signal {
    Reset,
    Start,
    Stop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cradle() {
        struct BabyImpl {
            times: usize,
        }
        impl Baby for BabyImpl {
            // Only cry `times`, and cry when elapsed time >= 2.
            fn cry(&mut self, elapsed: usize) -> BoxResult<()> {
                if elapsed >= 2 && self.times > 0 {
                    self.times -= 1;
                    println!("Baby cries at {}", elapsed);
                }
                Ok(())
            }
        }
        let cradle = Cradle::new(vec![BabyImpl { times: 2 }]);
        cradle.start();
        thread::sleep(Duration::from_secs(4));
        // The baby should cry twice.
        cradle.stop();
        cradle.join().unwrap().unwrap();
    }
}
