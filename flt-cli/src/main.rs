// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

// Main function
fn main() {
    println!("flt version {}", version());
}
