fn main() {
    // create an array of fixed size
    let a = [76, 77, 78, 79, 80];
    // get slice which element indexes are in [1, 4) interval
    let b = &a[1..4];
    // print out using fmt of Debug trait
    println!("{b:?}");
}
