fn main() {
    // string literal (placed program file)
    let s1 = "hello";

    // heap allocated string
    let s2 = String::from("hello");

    // slice of the whole length
    let s3 = s2.as_str();

    // Dereference to str:
    //   size_of_val takes &T
    //   => s1 = &str = &T
    //   => T = str.
    // Value equals to length of slice because each element is u8.
    let size_of_s1 = std::mem::size_of_val(s1);

    // dereference to String,
    // which consists of 3 parts x usize:
    // - pointer to start
    // - length
    // - capacity
    let size_of_s2 = std::mem::size_of_val(&s2);

    // dereference to &str, which is fat pointer:
    // 2 parts x usize: pointer to start + length
    let size_of_s3 = std::mem::size_of_val(&s3);

    println!("{:?}", size_of_s1);
    println!("{:?}", size_of_s2);
    println!("{:?}", size_of_s3);
}
