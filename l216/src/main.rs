fn as_chan(vs: &[i32]) -> std::sync::mpsc::Receiver<i32> {
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn({
        // Own values to pass them to another thread.
        // The owned slice is a vector.
        let vs = vs.to_owned();

        move || {
            for v in vs {
                // the receiver (rx) doesn't consume any value before `join`,
                // so `send` pushes each value into the channel inner queue
                // and does this in the same order as the iterator over `vs`
                tx.send(v).unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1))
            }

            // Drop sender (tx) immidiatly to allow the recevier consuming all values
            // and getting an error when the queue gets empty.
            //
            // In the current implementation this call doesn't affect on anything,
            // since `tx` is moved regardless of calling `drop`
            // due to move keyword and use of `send`, which requires &tx.
            drop(tx);
        }
    });

    // Block the current thread untill the worker is done.
    //
    // Even though `as_chan` is called outside twice,
    // each call is processed synchroniusly due to `join`.
    handle.join().unwrap();

    rx
}

fn merge(
    a: std::sync::mpsc::Receiver<i32>,
    b: std::sync::mpsc::Receiver<i32>,
) -> std::sync::mpsc::Receiver<i32> {
    let (tx, rx) = std::sync::mpsc::channel();

    // a safe way to signal when the channel is empty
    let mut a_done = false;
    let mut b_done = false;

    // `try_recv` returns an error
    // if there are no more values in the channel queue.
    //
    // The both channels have the same length,
    // thus the new channel gets one value from `a` and one value from `b`
    // on each iteration untill `a` and `b` becomes empty (in the same iteration):
    //   rx queue is [] before the loop;
    //   loop: 0
    //     1 from a, 2 from b -> (queue behind rx) [1, 2]
    //   loop: 1
    //     3 from a, 4 from b -> [1, 2, 3, 4]
    //   ...
    //   loop: 3
    //     7 from a, 8 from b -> [1, 2, 3, 4, 5, 6, 7, 8]
    //   loop: 4
    //     a.try_recv returns Err(_), b.try_recv retruns Err(_)
    //     => a_done = true, b_done = true
    //     => break
    loop {
        match a.try_recv() {
            Ok(i) => {
                tx.send(i).unwrap();
            }

            Err(_) => {
                a_done = true;
            }
        }

        match b.try_recv() {
            Ok(i) => {
                tx.send(i).unwrap();
            }

            Err(_) => {
                b_done = true;
            }
        }

        if a_done && b_done {
            break;
        }
    }

    rx
}

fn main() {
    // Put values in an unbounded channel and return its receiver.
    // Each value stays in the channel untill `try_recv` has been called.
    // At the end of these calls each recevier is ready to take values from its queue (FIFO):
    // - queue behind receiver `a` is [1, 3, 5, 7],
    // - queue behind receiver `b` is [2, 4, 6, 8].
    let a = as_chan(&vec![1, 3, 5, 7]);
    let b = as_chan(&vec![2, 4, 6, 8]);

    // at this point in about 8 seconds from the beginning:
    // each `as_chan` call takes about 4 seconds

    // Create a new unbounded channel from the previous two.
    // The new channel queue contains values in the following order:
    // 1, 2, 3, 4, 5, 6, 7, 8.
    // See datails in `merge` body.
    let c = merge(a, b);

    // consume each value from `c`: 1, 2, etc.
    for v in c.iter() {
        println!("{v:?}");
    }
}
