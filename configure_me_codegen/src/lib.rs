//! This is the codegen part of `configure_me` crate. Please refer to the documentation of
//! `configure_me`.

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
/*
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
*/
extern crate unicode_segmentation;
extern crate fmt2io;
extern crate cargo_toml;
#[cfg(feature = "man")]
extern crate man;

pub(crate) mod config;
pub(crate) mod codegen;
#[cfg(feature = "man")]
pub (crate) mod gen_man;
#[cfg(feature = "debconf")]
pub (crate) mod debconf;

pub mod manifest;

use std::borrow::Borrow;
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use manifest::LoadManifest;

#[derive(Debug)]
enum ErrorData {
    Toml(toml::de::Error),
    Config(config::ValidationError),
    Io(io::Error),
    Open { file: PathBuf, error: io::Error },
    Manifest(manifest::Error),
    MissingManifestDirEnvVar,
    MissingOutDir,
    #[cfg(feature = "debconf")]
    Debconf(debconf::Error),
}

/// Error that occured during code generation
pub struct Error {
    data: ErrorData,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.data {
            ErrorData::Toml(err) => write!(f, "failed to parse config specification: {}", err),
            ErrorData::Manifest(error) => write!(f, "failed to process manifest: {}", error),
            ErrorData::Config(err) => fmt::Display::fmt(err, f),
            ErrorData::Io(err) => write!(f, "I/O error: {}", err),
            ErrorData::Open { file, error } => write!(f, "failed to open file {}: {}", file.display(), error),
            ErrorData::MissingManifestDirEnvVar => write!(f, "missing environment variable: CARGO_MANIFEST_DIR"),
            ErrorData::MissingOutDir => write!(f, "missing environment variable: OUT_DIR"),
            #[cfg(feature = "debconf")]
            ErrorData::Debconf(err) => write!(f, "failed to generate debconf: {}", err),
        }
    }
}

/// Implemented using `Display` so that it can be used with `Termination` to display nicer message.
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
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

impl From<manifest::Error> for Error {
    fn from(err: manifest::Error) -> Self {
        Error {
            data: ErrorData::Manifest(err),
        }
    }
}

impl From<manifest::LoadError> for Error {
    fn from(err: manifest::LoadError) -> Self {
        Error {
            data: ErrorData::Manifest(err.into()),
        }
    }
}

impl From<void::Void> for Error {
    fn from(value: void::Void) -> Self {
        match value {}
    }
}


#[cfg(feature = "debconf")]
impl From<debconf::Error> for Error {
    fn from(err: debconf::Error) -> Self {
        Error {
            data: ErrorData::Debconf(err),
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

fn load_from_file<P: AsRef<Path>>(source: P) -> Result<::config::Config, Error> {
     let config_spec = std::fs::File::open(&source).map_err(|error| ErrorData::Open { file: source.as_ref().into(), error })?;

     load(config_spec)
}

fn path_in_out_dir<P: AsRef<Path>>(file_name: P) -> Result<PathBuf, Error> {
    let mut out: PathBuf = std::env::var_os("OUT_DIR").ok_or(ErrorData::MissingOutDir)?.into();
    out.push(file_name);

    Ok(out)
}

fn default_out_file(binary: Option<&str>) -> Result<PathBuf, Error> {
    const GENERATED_FILE_NAME: &str = "configure_me_config.rs";

    let file_name_owned;
    let file_name = match binary {
        Some(binary) => {
            file_name_owned = format!("{}_{}", binary, GENERATED_FILE_NAME);
            &file_name_owned
        },
        None => GENERATED_FILE_NAME,
    };
    path_in_out_dir(file_name)
}

// Wrapper for error conversions
fn create_file<P: AsRef<Path> + Into<PathBuf>>(file: P) -> Result<std::fs::File, Error> {
    std::fs::File::create(&file)
        .map_err(|error| ErrorData::Open { file: file.into(), error })
        .map_err(Into::into)
}

fn generate_to_file<P: AsRef<Path> + Into<PathBuf>>(config_spec: &::config::Config, file: P) -> Result<(), Error> {
     let config_code = create_file(file)?;
     ::fmt2io::write(config_code, |config_code| codegen::generate_code(config_spec, config_code)).map_err(Into::into)
}

fn load_and_generate_default<P: AsRef<Path>>(source: P, binary: Option<&str>) -> Result<::config::Config, Error> {
    let config_spec = load_from_file(&source)?;
    generate_to_file(&config_spec, default_out_file(binary)?)?;
    #[cfg(feature = "debconf")]
    debconf::generate_if_requested(&config_spec)?;
    println!("cargo:rerun-if-changed={}", source.as_ref().display());
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
pub fn build_script<P: AsRef<Path>>(source: P) -> Result<(), Error> {
    load_and_generate_default(source, None).map(::std::mem::drop)
}

/// Generates the source code for you
///
/// Finds the specification in Cargo.toml `metadata.configure_me`
///
/// This function should be used from build script as it relies on cargo environment. It handles
/// generating the name of the file (it's called `config.rs` inside `OUT_DIR`) as well as notifying
/// cargo of the source file.
pub fn build_script_auto() -> Result<(), Error> {
    use manifest::SpecificationPaths;

    let manifest_dir = manifest::get_dir()?;
    let manifest_file = manifest_dir.join("Cargo.toml");

    let paths = manifest_file
        .load_manifest()?
        .package.ok_or(manifest::Error::MissingPackage)?
        .metadata.ok_or(manifest::Error::MissingMetadata)?
        .configure_me.ok_or(manifest::Error::MissingConfigureMeMetadata)?
        .spec_paths;

    match paths {
        SpecificationPaths::Single(path) => load_and_generate_default(manifest_dir.join(path), None).map(::std::mem::drop),
        SpecificationPaths::PerBinary(binaries) => {
            for (binary, path) in binaries {
                load_and_generate_default(manifest_dir.join(path), Some(&binary)).map(::std::mem::drop)?;
            }
            Ok(())
        },
        SpecificationPaths::Other(other) => match other._private {},
    }
}

/// Generates the source code and manual page at default location.
///
/// This is same as `build_script()`, but additionaly it generates a man page.
/// The resulting man page will be stored in `$OUT_DIR/app.man`.
#[cfg(feature = "man")]
pub fn build_script_with_man<P: AsRef<Path>>(source: P) -> Result<(), Error> {
    build_script_with_man_written_to(source, path_in_out_dir("app.man")?)
}

/// Generates the source code and manual page at specified location.
///
/// This is same as `build_script_with_man()`, but it allows you to choose where to put the man
/// page.
#[cfg(feature = "man")]
pub fn build_script_with_man_written_to<P: AsRef<Path>, M: AsRef<Path> + Into<PathBuf>>(source: P, output: M) -> Result<(), Error> {
    let config_spec = load_and_generate_default(source, None)?;
    let manifest = manifest::BuildScript.load_manifest()?;
    let man_page = gen_man::generate_man_page(&config_spec, manifest.borrow())?;

    let mut file = create_file(output)?;
    file.write_all(man_page.as_bytes())?;
    #[cfg(feature = "debconf")]
    debconf::generate_if_requested(&config_spec)?;
    Ok(())
}

/// Generates man page **only**.
///
/// This is useful outside build scripts.
#[cfg(feature = "man")]
pub fn generate_man<M: LoadManifest, W: std::io::Write, S: AsRef<Path>>(source: S, mut dest: W, manifest: M) -> Result<(), Error> where Error: std::convert::From<<M as manifest::LoadManifest>::Error> {
    let config_spec = load_from_file(&source)?;
    let manifest = manifest.load_manifest()?;
    let man_page = gen_man::generate_man_page(&config_spec, manifest.borrow())?;
    dest.write_all(man_page.as_bytes())?;
    Ok(())
}

#[cfg(test)]
#[deny(warnings)]
pub(crate) mod tests {
    use ::generate_source;

    pub const SINGLE_OPTIONAL_PARAM: &str =
r#"
[general]
env_prefix = "TEST_APP"

[[param]]
name = "foo"
type = "u32"
"#;

    pub const SINGLE_MANDATORY_PARAM: &str =
r#"
[general]
env_prefix = "TEST_APP"

[[param]]
name = "foo"
type = "u32"
optional = false
"#;

    pub const SINGLE_DEFAULT_PARAM: &str =
r#"
[general]
env_prefix = "TEST_APP"

[[param]]
name = "foo"
type = "u32"
default = "42"
"#;

    pub const SINGLE_SWITCH: &str =
r#"
[general]
env_prefix = "TEST_APP"

[[switch]]
name = "foo"
"#;

    pub const MULTIPLE_PARAMS: &str =
r#"
[general]
env_prefix = "TEST_APP"

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
[general]
env_prefix = "TEST_APP"

[[param]]
name = "foo"
type = "u32"
argument = false
"#;

    pub const SHORT_SWITCHES: &str =
r#"
[[switch]]
name = "a"
abbr = "a"
doc = "test"

[[switch]]
name = "b"
abbr = "b"

[[switch]]
name = "c"
abbr = "c"
count = true

[[param]]
name = "d"
type = "String"
abbr = "d"
optional = true

[[param]]
name = "e"
type = "String"
abbr = "e"
optional = true

[[switch]]
name = "foo_bar"
abbr = "f"
"#;

    pub const CONF_FILES: &str =
r#"
[general]
env_prefix = "TEST_APP"
conf_file_param = "config"
conf_dir_param = "conf_dir"

[[param]]
name = "foo"
type = "u32"
doc = "A foo"
"#;

    pub const CUSTOM_MERGE_FN: &str =
r#"
[general]
env_prefix = "TEST_APP"

[[param]]
name = "foo"
type = "u32"
merge_fn = "(|a: &mut u32, b: u32| *a += b)"

[[param]]
name = "bar"
type = "String"
merge_fn = "(|a: &mut String, b: String| a.push_str(&b))"
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

    pub const EXPECTED_SHORT_SWITCHES: ExpectedOutput = ExpectedOutput {
        raw_config: include_str!("../tests/expected_outputs/short_switches/raw_config.rs"),
        validate: include_str!("../tests/expected_outputs/short_switches/validate.rs"),
        merge_in: include_str!("../tests/expected_outputs/short_switches/merge_in.rs"),
        merge_args: include_str!("../tests/expected_outputs/short_switches/merge_args.rs"),
        config: include_str!("../tests/expected_outputs/short_switches/config.rs"),
        arg_parse_error: include_str!("../tests/expected_outputs/short_switches/arg_parse_error.rs"),
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

    #[test]
    fn short_switches() {
        check(SHORT_SWITCHES, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/short_switches-config.rs")));
    }

    #[test]
    fn conf_files() {
        check(CONF_FILES, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/conf_files-config.rs")));
    }

    #[test]
    fn custom_merge_fn() {
        check(CUSTOM_MERGE_FN, include_str!(concat!(env!("OUT_DIR"), "/expected_outputs/with_custom_merge-config.rs")));
    }
}
