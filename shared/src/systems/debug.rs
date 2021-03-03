use legion::system;

#[system]
pub fn tick_debug() {
    println!("tick");
}
