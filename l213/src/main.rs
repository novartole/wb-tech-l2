struct Example(i32);

impl Drop for Example {
    // .. --> this now contains 0
    // due to call of replace in its parent.
    //
    // > 0
    //
    fn drop(&mut self) {
        println!("{}", self.0);
    }
}

struct ExampleWrap(Example);

impl Drop for ExampleWrap {
    // This drop is called first.
    // _e_ takes value of self.0 (~ 8) and holds it till the end of scope.
    // After this drop, drop of nested structure is called --> ..
    //
    // > warp 8
    // > 8
    //
    fn drop(&mut self) {
        let e = std::mem::replace(&mut self.0, Example(0));
        println!("wrap {}", e.0);
    }
}

fn main() {
    // drop immediately
    Example(1);

    // these two live till the end of scope
    // and will be dropped in reverse order: 3, 2
    let _e2 = Example(2);
    let _e3 = Example(3);

    // drop immediately
    let _ = Example(4);

    let mut _e5;
    _e5 = Some(Example(5));
    // this drop the previous value - drop 5 is called immediately
    _e5 = None;

    let e6 = Example(6);
    // force to call drop
    drop(e6);

    let e7 = Example(7);
    // drop won't be called,
    // value lives till the end of program;
    // ownership of e7 is taken by _forget_,
    // so it cannot be used in future
    std::mem::forget(e7);
    println!("here");
    // wrapper's drop is called first,
    // then it's turn of nested drop
    ExampleWrap(Example(8));
}
