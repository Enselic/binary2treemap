fn add<T>(a: T, b: T) -> T
where
    T: std::ops::Add<Output = T>,
{
    a + b
}

fn main() {
    let i = u32::from_str_radix(&std::env::args().next().unwrap(), 10).unwrap();
    let x = add(i, 100u32);
    let y = add(i as u128, 100u128);
    println!("x = {x}, y = {y}");
}


