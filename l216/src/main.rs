fn as_chan(vs: &[i32]) -> std::sync::mpsc::Receiver<i32> {
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn({
        // clone values into new vec
        let vs = vs.to_owned();

        move || {
            for v in vs {
                tx.send(v).unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1))
            }

            drop(tx);
        }
    });

    // sync to return kept values within rx
    handle.join().unwrap();

    rx
}

fn merge(
    a: std::sync::mpsc::Receiver<i32>,
    b: std::sync::mpsc::Receiver<i32>,
) -> std::sync::mpsc::Receiver<i32> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut a_done = false;

    let mut b_done = false;

    // _try_recv_ returns error in case if there are no more values in buffer
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
    // put values into unbounded channel buffer
    let a = as_chan(&vec![1, 3, 5, 7]);
    // second starts a bit leter, so merge will take them off in right order
    let b = as_chan(&vec![2, 4, 6, 8]);
    // take them back and put into another buffer;
    // new buffer contains values of both previous slices
    let c = merge(a, b);

    // take everything back from buffer
    for v in c.iter() {
        println!("{v:?}");
    }
}
