macro_rules! test_name { () => { "single_optional_param" } }

include!("glue/boilerplate.rs");

#[test]
fn main() {
    use std::iter;
    use std::path::PathBuf;

    let _ = config::Config::including_optional_config_files(iter::empty::<PathBuf>()).unwrap();
}
