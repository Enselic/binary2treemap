fn add<T>(a: T, b: T) -> T
where
    T: std::ops::Add<Output = T>,
{
    a + b
}

fn main() {
    let x = add(1u8, 2u8);
    let y = add(100u128, 200u128);
    println!("x = {x}, y = {y}");
}
