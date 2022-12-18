macro_rules! test_name { () => { "conf_files" } }

include!("glue/boilerplate.rs");

#[test]
fn config_ordering() {
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
    let empty = this.join("empty.toml");
    let fortytwo = this.join("fortytwo.toml");
    let fortyseven = this.join("fortyseven.toml");
    let empty_args: &[&str] = &[];

    let (config, _, _) = config::Config::including_optional_config_files(&[&empty, &empty]).unwrap();
    assert!(config.foo.is_none());
    let (config, _, _) = config::Config::including_optional_config_files(&[&empty, &fortytwo]).unwrap();
    assert_eq!(config.foo, Some(42));
    let (config, _, _) = config::Config::including_optional_config_files(&[&fortytwo, &empty]).unwrap();
    assert_eq!(config.foo, Some(42));
    let (config, _, _) = config::Config::including_optional_config_files(&[&fortytwo, &fortytwo]).unwrap();
    assert_eq!(config.foo, Some(42));
    let (config, _, _) = config::Config::including_optional_config_files(&[&fortytwo, &fortyseven]).unwrap();
    assert_eq!(config.foo, Some(42));
    let (config, _, _) = config::Config::including_optional_config_files(&[&fortyseven, &fortytwo]).unwrap();
    assert_eq!(config.foo, Some(47));

    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--foo=42"], empty_args).unwrap();
    assert_eq!(config.foo, Some(42));
    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test", "--foo=42"], &[&fortyseven]).unwrap();
    assert_eq!(config.foo, Some(42));
    let (config, _, _) = config::Config::custom_args_and_optional_files(&["test".as_ref(), "--foo=50".as_ref(), "--config".as_ref(), fortytwo.as_path()], &[&fortyseven]).unwrap();
    assert_eq!(config.foo, Some(42));
}
