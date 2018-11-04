macro_rules! test_name { () => { "single_mandatory_param" } }

include!("glue/boilerplate.rs");

use std::iter;
use std::path::PathBuf;

#[test]
fn no_args() {
    let result = config::Config::custom_args_and_optional_files(&["display"], iter::empty::<PathBuf>());
    match result {
        Ok(_) => panic!("This shouldn't succeed"),
        Err(err) => assert_eq!(err.to_string(), "Invalid configuration: Configuration parameter 'foo' not specified."),
    }
}

#[test]
fn missing_arg() {
    let result = config::Config::custom_args_and_optional_files(&["display", "--foo"], iter::empty::<PathBuf>());
    match result {
        Ok(_) => panic!("This shouldn't succeed"),
        Err(err) => assert_eq!(err.to_string(), "A value to argument '--foo' is missing."),
    }
}

#[test]
fn parse_fail() {
    let result = config::Config::custom_args_and_optional_files(&["display", "--foo", "fortytwo"], iter::empty::<PathBuf>());
    match result {
        Ok(_) => panic!("This shouldn't succeed"),
        Err(err) => assert_eq!(err.to_string(), "Failed to parse argument '--foo': invalid digit found in string."),
    }
}

#[test]
fn unknown_arg() {
    let result = config::Config::custom_args_and_optional_files(&["display", "--bar"], iter::empty::<PathBuf>());
    match result {
        Ok(_) => panic!("This shouldn't succeed"),
        Err(err) => assert_eq!(err.to_string(), "An unknown argument '--bar' was specified."),
    }
}
