macro_rules! test_name { () => { "single_optional_param" } }

include!("glue/boilerplate.rs");

#[test]
fn main() {
    use std::iter;
    use std::path::PathBuf;

    let _ = config::Config::custom_args_and_optional_files(&["custom_args", "--foo", "42"], iter::empty::<PathBuf>()).unwrap();
}
