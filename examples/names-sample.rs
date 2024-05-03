#[path ="shared/lib.rs"]
mod shared;
use shared::random_name;

fn main() {
    for _ in 0..1000 {
        println!("{}", random_name());
    }
}
