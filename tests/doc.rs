macro_rules! test_name { () => { "multiple_params" } }

include!("glue/boilerplate.rs");

use std::iter;
use std::path::PathBuf;

const EXPECTED_HELP: &str = r##"Usage: miner [--foo FOO] [--bar BAR] [--baz BAZ] [--verbose] [--no-fast]
Arguments:
        --foo        A foo
        --bar        A very, very, very, very, very, very, very, very, very, 
                     very, very, very, very, very long documentation...
        --baz        A much, much, much, much, much, much, much, much, much, 
                     much, much, much, much, much, much, much, much, much, much,
                     much, much, much, much, much, much, much, much, much, much,
                     much, much, much, much, much, much, much, much, much, much,
                     much, much, much longer documentation...
        --no-fast    Determines whether to mine bitcoins fast or slowly"##;

#[test]
fn help_multiple_params() {
    let result = config::Config::custom_args_and_optional_files(&["miner", "--help"], iter::empty::<PathBuf>());
    match result {
        Ok(_) => panic!("This shouldn't succeed"),
        Err(err) => assert_eq!(err.to_string(), EXPECTED_HELP),
    }
}

#[test]
#[ignore]
fn process_help() {
    use config::prelude::*;
    let _ = Config::custom_args_and_optional_files(&["miner", "--help"], iter::empty::<PathBuf>()).unwrap_or_exit();
}

#[test]
#[ignore]
fn process_error() {
    use config::prelude::*;
    let _ = Config::custom_args_and_optional_files(&["miner", "--foo"], iter::empty::<PathBuf>()).unwrap_or_exit();
}
