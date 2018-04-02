extern crate vergen;
use vergen::vergen;

fn main() {
    let flags = vergen::OutputFns::all();
    // Generate the version.rs file in the Cargo OUT_DIR.
    assert!(vergen(flags).is_ok());
}
