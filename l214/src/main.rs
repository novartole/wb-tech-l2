fn main() {
    // create unbounded channel
    let (tx, rv) = std::sync::mpsc::channel::<i32>();

    // _move_ because there is no garantee
    // that tx outlives spawned thread
    let handle = std::thread::spawn(move || {
        for i in 0..10 {
            tx.send(i).unwrap();
        }
    });

    // Wait for handle is done it task.
    // Values are kept by channel buffer.
    handle.join().unwrap();

    // get all values from buffer
    for i in rv.iter() {
        println!("{i:?}");
    }
}
