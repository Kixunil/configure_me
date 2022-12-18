macro_rules! test_name { () => { "with_custom_merge" } }

include!("glue/boilerplate.rs");

#[test]
fn custom_merge_fn() {
    use std::path::PathBuf;

    let mut this = PathBuf::from(std::env::args_os().next().expect("Program name not specified"));

    while let Some(file_name) = this.file_name() {
        if *file_name == *"target" {
            break;
        }

        this.pop();
    }

    if !this.pop() {
        panic!("Can't find test assets");
    }

    this.push("configure_me_codegen");
    if !this.exists() {
        this.pop();
    }
    this.push("tests");
    this.push("config_files");
    let fortytwo = this.join("fortytwo.toml");
    let bar_hello = this.join("bar_hello.toml");
    let empty_args: &[&str] = &[];

    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--foo=42"], empty_args).unwrap();
    assert_eq!(config.foo, Some(42));
    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--foo=42", "--foo=5"], empty_args).unwrap();
    assert_eq!(config.foo, Some(47));
    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--foo=5"], &[fortytwo]).unwrap();
    assert_eq!(config.foo, Some(47));
    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--bar=Hello"], empty_args).unwrap();
    assert_eq!(config.bar.as_ref().map(AsRef::as_ref), Some("Hello"));
    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--bar=Hello", "--bar= world"], empty_args).unwrap();
    assert_eq!(config.bar.as_ref().map(AsRef::as_ref), Some("Hello world"));
    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--bar= world"], &[bar_hello]).unwrap();
    assert_eq!(config.bar.as_ref().map(AsRef::as_ref), Some("Hello world"));
}
