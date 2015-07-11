use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

static NTHREADS: i32 = 3;

enum Events<T: 'static> {
    Task(Box<Fn()->bool+Send>),
    Data(T)
}

struct Fiber<T: 'static> {
    sender: Sender<Events<T>>,
    t: std::thread::JoinHandle<()>
}

impl <T: Send> Fiber<T> {
    fn new<F>(fun: F) -> Fiber<T>
        where  F: Send + 'static + Fn(T),{
        let (tx, rx): (Sender<Events<T>>, Receiver<Events<T>>) = mpsc::channel();
        let t = thread::spawn (move || {
            let mut running = true;
            while running {
                let event = rx.recv().unwrap();
                match event {
                    Events::Task(t) => running = t(),
                    Events::Data(d) => fun(d)
                }
            }
        });
        return Fiber{sender:tx, t:t};
    }

    fn stop(self, wait_for_thread_end: bool) {
        let end = move || {
            return false;
        };
        self.sender.send(Events::Task(Box::new(end))).unwrap();
        if wait_for_thread_end {
            self.join();
        }
    }
    fn join(self) {
        self.t.join().unwrap();
    }
}

fn main() {
    let mut vec = Vec::new();
    for id in 0..NTHREADS {
        let rcv_loop = |data: i32| {
            println!("{:?}", data);
        };
        let f: (Fiber<i32>) = Fiber::new(rcv_loop);
        let printer = move || {
            println!("{:?}", id);
            return true;
        };
        f.sender.send(Events::Task(Box::new(printer))).unwrap();
        f.sender.send(Events::Data(id + 1000)).unwrap();
        vec.push(f);
    }

    for f in vec {
        f.stop(true);
    }
}