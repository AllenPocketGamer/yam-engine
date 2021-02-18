fn main() {
    let num_logical = num_cpus::get();
    println!("logical: {}", num_logical);
    let num_physical = num_cpus::get_physical();
    println!("physical: {}", num_physical);
}