extern crate vergen;

fn main() {
    let flags = vergen::ConstantsFlags::all();
    // Generate the version.rs file in the Cargo OUT_DIR.
    assert!(vergen::generate_version_rs(flags).is_ok());
}
