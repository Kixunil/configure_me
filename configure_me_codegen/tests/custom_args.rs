macro_rules! test_name { () => { "single_optional_param" } }

include!("glue/boilerplate.rs");

use std::iter;
use std::path::{PathBuf, Path};

#[test]
fn custom_args() {
    let (config, mut remaining, metadata) = config::Config::custom_args_and_optional_files(&["custom_args", "--foo", "42"], iter::empty::<PathBuf>()).unwrap();
    assert_eq!(metadata.program_name.unwrap(), Path::new("custom_args"));
    assert_eq!(config.foo, Some(42));
    assert_eq!(remaining.next(), None);
}

#[test]
fn custom_args_with_one_remaining() {
    let (config, mut remaining, metadata) = config::Config::custom_args_and_optional_files(&["custom_args", "--foo", "42", "remaining_arg"], iter::empty::<PathBuf>()).unwrap();
    assert_eq!(metadata.program_name.unwrap(), Path::new("custom_args"));
    assert_eq!(config.foo, Some(42));
    assert_eq!(remaining.next(), Some("remaining_arg".into()));
    assert_eq!(remaining.next(), None);
}

#[test]
fn custom_args_with_two_remaining() {
    let (config, mut remaining, metadata) = config::Config::custom_args_and_optional_files(&["custom_args", "--foo", "42", "remaining_arg", "another_arg"], iter::empty::<PathBuf>()).unwrap();
    assert_eq!(metadata.program_name.unwrap(), Path::new("custom_args"));
    assert_eq!(config.foo, Some(42));
    assert_eq!(remaining.next(), Some("remaining_arg".into()));
    assert_eq!(remaining.next(), Some("another_arg".into()));
    assert_eq!(remaining.next(), None);
}

#[test]
fn custom_args_with_two_dashes() {
    let (config, mut remaining, metadata) = config::Config::custom_args_and_optional_files(&["custom_args", "--", "--foo", "42"], iter::empty::<PathBuf>()).unwrap();
    assert_eq!(metadata.program_name.unwrap(), Path::new("custom_args"));
    assert_eq!(config.foo, None);
    assert_eq!(remaining.next(), Some("--foo".into()));
    assert_eq!(remaining.next(), Some("42".into()));
    assert_eq!(remaining.next(), None);
}

#[test]
fn custom_args_with_two_dashes_foo_equals_val() {
    let (config, mut remaining, metadata) = config::Config::custom_args_and_optional_files(&["custom_args", "--", "--foo=42"], iter::empty::<PathBuf>()).unwrap();
    assert_eq!(metadata.program_name.unwrap(), Path::new("custom_args"));
    assert_eq!(config.foo, None);
    assert_eq!(remaining.next(), Some("--foo=42".into()));
    assert_eq!(remaining.next(), None);
}

#[test]
fn param_equals_value() {
    let (config, mut remaining, metadata) = config::Config::custom_args_and_optional_files(&["custom_args", "--foo=42"], iter::empty::<PathBuf>()).unwrap();
    assert_eq!(metadata.program_name.unwrap(), Path::new("custom_args"));
    assert_eq!(config.foo, Some(42));
    assert_eq!(remaining.next(), None);
}
