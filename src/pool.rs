use std::sync::{atomic::AtomicBool, Arc, Condvar, Mutex};

use std::collections::VecDeque;
use std::marker::Send;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Pool<T>
where
    T: Send,
{
    terminal: Arc<AtomicBool>,
    pub tasks: Arc<(Mutex<VecDeque<T>>, Condvar)>,
    threads: Vec<JoinHandle<()>>,
}

impl<T> Pool<T>
where
    T: Send,
{
    pub fn new<F>(size: usize, execution: &'static F) -> Pool<T>
    where
        T: Send + 'static,
        F: Fn(T) + Send + Sync,
    {
        let singal = Arc::new(AtomicBool::new(false));
        let channel = Arc::new(((Mutex::new(VecDeque::new())), Condvar::new()));

        let threads = vec![];
        for _iter in 0..size {
            let channel = channel.clone();

            let execution = execution;
            thread::spawn(move || loop {
                let channel = &*channel;
                // let (ref a, ref b)=*channel;
                let mut queue = channel.0.lock().unwrap();
                while queue.len() == 0 {
                    queue = channel.1.wait(queue).unwrap();
                }
                match queue.pop_front() {
                    Some(val) => {
                        drop(queue);
                        execution(val);
                    }
                    None => {}
                }
            });
        }

        Pool {
            terminal: singal,
            tasks: channel,
            threads,
        }
    }
    pub fn execute(&mut self, content: T) {
        let mut channel = self.tasks.0.lock().unwrap();
        channel.push_back(content);
        self.tasks.1.notify_one();
    }
}

mod test {
    use super::*;
    use std::{
        sync::mpsc::{self, SyncSender},
        thread::sleep,
    };

    #[test]
    fn pool_test() {
        let (tx, rx) = mpsc::sync_channel::<usize>(100);

        struct Job {
            context: usize,
            dur: usize,
        }
        fn execution(inp: (SyncSender<usize>, Job)) {
            let (tx, a) = inp;
            sleep(Duration::from_millis(a.dur as u64));
            tx.send(a.context).unwrap();
        }

        let mut pool = Pool::new(10, &execution);

        sleep(Duration::from_millis(300));

        for iter in 0..100 {
            pool.execute((
                tx.clone(),
                Job {
                    context: iter,
                    dur: iter,
                },
            ));
        }

        let mut rx_result = Vec::with_capacity(100);
        for iter in 0..100 {
            rx_result.push(rx.recv().unwrap());
        }

        rx_result.sort();

        let expect_result: Vec<usize> = (0..100).collect();

        assert_eq!(rx_result, expect_result);
    }
}
