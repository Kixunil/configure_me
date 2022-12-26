macro_rules! test_name { () => { "optional_program_name" } }

include!("glue/boilerplate.rs");

use std::iter;
use std::path::{Path, PathBuf};

#[test]
fn custom_args() {
    let (_config, _remaining, metadata) = config::Config::custom_args_and_optional_files(&["custom_args"], iter::empty::<PathBuf>()).unwrap();
    assert_eq!(metadata.program_name.unwrap(), Path::new("custom_args"))
}

