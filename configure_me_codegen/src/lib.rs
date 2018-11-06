//! This is the codegen part of `configure_me` crate. Please refer to the documentation of
//! `configure_me`.

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate unicode_segmentation;
extern crate fmt2io;
extern crate build_helper;
#[cfg(feature = "man")]
extern crate man;

pub(crate) mod config;
pub(crate) mod codegen;
#[cfg(feature = "man")]
pub (crate) mod gen_man;

use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Debug)]
enum ErrorData {
    Toml(toml::de::Error),
    Config(config::ValidationError),
    Io(io::Error),
    Open { file: PathBuf, error: io::Error },
    MissingOutDir,
}

/// Error that occured during code generation
///
/// It currently only implements Debug, which should
/// be sufficient for now. This may be improved in the future.
#[derive(Debug)]
pub struct Error {
    data: ErrorData,
}

impl From<ErrorData> for Error {
    fn from(data: ErrorData) -> Self {
        Error {
            data,
        }
    }
}

impl From<config::ValidationError> for Error {
    fn from(err: config::ValidationError) -> Self {
        Error {
            data: ErrorData::Config(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error {
            data: ErrorData::Io(err),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error {
            data: ErrorData::Toml(err),
        }
    }
}

fn load<S: Read>(mut source: S) -> Result<config::Config, Error> {
    let mut data = Vec::new();
    source.read_to_end(&mut data)?;
    let cfg = toml::from_slice::<config::raw::Config>(&data)?;
    let cfg = cfg.validate()?;

    Ok(cfg)
}

fn load_from_file<P: AsRef<std::path::Path>>(source: P) -> Result<::config::Config, Error> {
     let config_spec = std::fs::File::open(&source).map_err(|error| ErrorData::Open { file: source.as_ref().into(), error })?;

     load(config_spec)
}

fn path_in_out_dir<P: AsRef<std::path::Path>>(file_name: P) -> Result<PathBuf, Error> {
    let mut out: PathBuf = std::env::var_os("OUT_DIR").ok_or(ErrorData::MissingOutDir)?.into();
    out.push(file_name);

    Ok(out)
}

fn default_out_file() -> Result<PathBuf, Error> {
    path_in_out_dir("configure_me_config.rs")
}

// Wrapper for error conversions
fn create_file<P: AsRef<std::path::Path> + Into<PathBuf>>(file: P) -> Result<std::fs::File, Error> {
    std::fs::File::create(&file)
        .map_err(|error| ErrorData::Open { file: file.into(), error })
        .map_err(Into::into)
}

fn generate_to_file<P: AsRef<std::path::Path> + Into<PathBuf>>(config_spec: &::config::Config, file: P) -> Result<(), Error> {
     let config_code = create_file(file)?;
     ::fmt2io::write(config_code, |config_code| codegen::generate_code(config_spec, config_code)).map_err(Into::into)
}

fn load_and_generate_default<P: AsRef<std::path::Path>>(source: P) -> Result<::config::Config, Error> {
    let config_spec = load_from_file(&source)?;
    generate_to_file(&config_spec, default_out_file()?)?;
    println!("rerun-if-changed={}", source.as_ref().display());
    Ok(config_spec)
}

/// Generates the source code for you from provided `toml` configuration.
pub fn generate_source<S: Read, O: Write>(source: S, output: O) -> Result<(), Error> {
    let cfg = load(source)?;
    
     ::fmt2io::write(output, |output| codegen::generate_code(&cfg, output)).map_err(Into::into)
}

/// Generates the source code for you from provided `toml` configuration file.
///
/// This function should be used from build script as it relies on cargo environment. It handles
/// generating the name of the file (it's called `config.rs` inside `OUT_DIR`) as well as notifying
/// cargo of the source file.
pub fn build_script<P: AsRef<std::path::Path>>(source: P) -> Result<(), Error> {
    load_and_generate_default(source).map(::std::mem::drop)
}

#[cfg(feature = "man")]
pub fn build_script_with_man<P: AsRef<std::path::Path>>(source: P) -> Result<(), Error> {
    let config_spec = load_and_generate_default(source)?;
    let man_page = gen_man::generate_man_page(&config_spec);

    let mut file = create_file(path_in_out_dir("app.man")?)?;
    file.write_all(man_page.as_bytes())?;
    Ok(())
}

#[cfg(test)]
#[deny(warnings)]
pub(crate) mod tests {
    use ::generate_source;

    pub const SINGLE_OPTIONAL_PARAM: &str =
r#"
[[param]]
name = "foo"
type = "u32"
"#;

    pub const SINGLE_MANDATORY_PARAM: &str =
r#"
[[param]]
name = "foo"
type = "u32"
optional = false
"#;

    pub const SINGLE_DEFAULT_PARAM: &str =
r#"
[[param]]
name = "foo"
type = "u32"
default = "42"
"#;

    pub const SINGLE_SWITCH: &str =
r#"
[[switch]]
name = "foo"
"#;

    pub const MULTIPLE_PARAMS: &str =
r#"
[[param]]
name = "foo"
type = "u32"
default = "42"
doc = "A foo"

[[param]]
name = "bar"
type = "String"
optional = true
doc = "A very, very, very, very, very, very, very, very, very, very, very, very, very, very long documentation..."

[[param]]
name = "baz"
type = "String"
optional = false
doc = "A much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much, much longer documentation..."

[[switch]]
name = "verbose"
# doc intentionally missing, because it's obious...

[[switch]]
name = "fast"
default = true
doc = "Determines whether to mine bitcoins fast or slowly"
"#;

    pub const NO_ARG: &str =
r#"
[[param]]
name = "foo"
type = "u32"
argument = false
"#;

    pub struct ExpectedOutput {
        pub raw_config: &'static str,
        pub validate: &'static str,
        pub merge_in: &'static str,
        pub merge_args: &'static str,
        pub config: &'static str,
        pub arg_parse_error: &'static str,
    }

    pub const EXPECTED_EMPTY: ExpectedOutput = ExpectedOutput {
        raw_config: include_str!("../tests/expected_outputs/empty/raw_config.rs"),
        validate: include_str!("../tests/expected_outputs/empty/validate.rs"),
        merge_in: include_str!("../tests/expected_outputs/empty/merge_in.rs"),
        merge_args: include_str!("../tests/expected_outputs/empty/merge_args.rs"),
        config: include_str!("../tests/expected_outputs/empty/config.rs"),
        arg_parse_error: include_str!("../tests/expected_outputs/empty/arg_parse_error.rs"),
    };

    pub const EXPECTED_SINGLE_OPTIONAL_PARAM: ExpectedOutput = ExpectedOutput {
        raw_config: include_str!("../tests/expected_outputs/single_optional_param/raw_config.rs"),
        validate: include_str!("../tests/expected_outputs/single_optional_param/validate.rs"),
        merge_in: include_str!("../tests/expected_outputs/single_optional_param/merge_in.rs"),
        merge_args: include_str!("../tests/expected_outputs/single_optional_param/merge_args.rs"),
        config: include_str!("../tests/expected_outputs/single_optional_param/config.rs"),
        arg_parse_error: include_str!("../tests/expected_outputs/single_optional_param/arg_parse_error.rs"),
    };

    pub const EXPECTED_SINGLE_MANDATORY_PARAM: ExpectedOutput = ExpectedOutput {
        raw_config: include_str!("../tests/expected_outputs/single_mandatory_param/raw_config.rs"),
        validate: include_str!("../tests/expected_outputs/single_mandatory_param/validate.rs"),
        merge_in: include_str!("../tests/expected_outputs/single_mandatory_param/merge_in.rs"),
        merge_args: include_str!("../tests/expected_outputs/single_mandatory_param/merge_args.rs"),
        config: include_str!("../tests/expected_outputs/single_mandatory_param/config.rs"),
        arg_parse_error: include_str!("../tests/expected_outputs/single_mandatory_param/arg_parse_error.rs"),
    };

    pub const EXPECTED_SINGLE_DEFAULT_PARAM: ExpectedOutput = ExpectedOutput {
        raw_config: include_str!("../tests/expected_outputs/single_default_param/raw_config.rs"),
        validate: include_str!("../tests/expected_outputs/single_default_param/validate.rs"),
        merge_in: include_str!("../tests/expected_outputs/single_default_param/merge_in.rs"),
        merge_args: include_str!("../tests/expected_outputs/single_default_param/merge_args.rs"),
        config: include_str!("../tests/expected_outputs/single_default_param/config.rs"),
        arg_parse_error: include_str!("../tests/expected_outputs/single_default_param/arg_parse_error.rs"),
    };

    pub const EXPECTED_SINGLE_SWITCH: ExpectedOutput = ExpectedOutput {
        raw_config: include_str!("../tests/expected_outputs/single_switch/raw_config.rs"),
        validate: include_str!("../tests/expected_outputs/single_switch/validate.rs"),
        merge_in: include_str!("../tests/expected_outputs/single_switch/merge_in.rs"),
        merge_args: include_str!("../tests/expected_outputs/single_switch/merge_args.rs"),
        config: include_str!("../tests/expected_outputs/single_switch/config.rs"),
        arg_parse_error: include_str!("../tests/expected_outputs/single_switch/arg_parse_error.rs"),
    };

    fn check(src: &str, expected: &str) {
        let mut src = src.as_bytes();
        let mut out = Vec::new();
        generate_source(&mut src, &mut out).unwrap();
        assert_eq!(::std::str::from_utf8(&out).unwrap(), expected);
    }

    #[test]
    fn empty() {
        check("", include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/empty-config.rs")));
    }

    #[test]
    fn single_optional_param() {
        check(SINGLE_OPTIONAL_PARAM, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/single_optional_param-config.rs")));
    }

    #[test]
    fn single_mandatory_param() {
        check(SINGLE_MANDATORY_PARAM, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/single_mandatory_param-config.rs")));
    }

    #[test]
    fn single_default_param() {
        check(SINGLE_DEFAULT_PARAM, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/single_default_param-config.rs")));
    }

    #[test]
    fn single_switch() {
        check(SINGLE_SWITCH, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/single_switch-config.rs")));
    }

    #[test]
    fn multiple_params() {
        check(MULTIPLE_PARAMS, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/multiple_params-config.rs")));
    }

    #[test]
    fn no_arg() {
        check(NO_ARG, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/no_arg-config.rs")));
    }
}
