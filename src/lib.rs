//! This library aims to help with reading configuration of application from files,
//! environment variables and command line arguments, merging it together and
//! validating. It auto-generates most of the code for you based on configuration (heh)
//! file. It creates a struct for you, which contains all the parsed and validated
//! fields, so you can access the information quickly easily and idiomatically.
//!
//! **Important note:** since this is generating code, it's intended to be used *from
//! build script*.
//!

//! Example
//! -------
//! 
//! Let's say, your application needs these parametrs to run:
//! 
//! * Port - this is mandatory
//! * IP address to bind to - defaults to 0.0.0.0
//! * Path to TLS certificate - optional, the server will be unsecure if not given
//! 
//! First you create Toml configuration file specifying all the parameters:
//! 
//! ```toml
//! [[param]]
//! name = "port"
//! type = "u16"
//! optional = false
//! 
//! [[param]]
//! name = "bind_addr"
//! # Yes, this works and  you can use your own T: Deserialize + FromStr as well!
//! type = "::std::net::Ipv4Addr" 
//! default = "::std::net::Ipv4Addr::new(0, 0, 0, 0)" # Rust expression that creates the value
//! 
//! [[param]]
//! name = "tls_cert"
//! type = "String"
//! # optional = true is the default, no need to add it here
//! # If the type is optional, it will be represented as Option<T>
//! # e.g. Option<String> in this case.
//! ```
//! 
//! Then, you create a build script like this:
//! 
//! ```rust,ignore
//! extern crate configure_me;
//! 
//! fn main() {
//!     let mut out: std::path::PathBuf = std::env::var_os("OUT_DIR").unwrap().into();
//!     out.push("config.rs");
//!     let config_spec = std::fs::File::open("config.toml").unwrap();
//!     let config_code = std::fs::File::create(&out).unwrap();
//!     configure_me::generate_source(config_spec, config_code).unwrap();
//!     println!("rerun-if-changed=config.toml");
//! }
//! ```
//! 
//! Add dependencies to `Cargo.toml`:
//! 
//! ```toml
//! [packge]
//! # ...
//! build = "build.rs"
//! 
//! [dependencies]
//! serde = "1"
//! serde_derive = "1"
//! toml = "0.4"
//! 
//! [build-dependencies]
//! configure_me = "0.1"
//! ```
//! 
//! Create a module `src/config.rs` for configuration:
//! 
//! ```rust,ignore
//! include!(concat!(env!("OUT_DIR"), "/config.rs"));
//! ```
//! 
//! And finally add appropriate incantiations into `src/main.rs`:
//! 
//! ```rust,ignore
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate toml;
//! 
//! mod config;
//! 
//! fn main() {
//!     // This will read configuration from "/etc/my_awesome_server/server.conf" file and
//!     // the command-line arguments.
//!     let server_config = config::Config::gather("/etc/my_awesome_server/server.conf").unwrap();
//! 
//!     // Your code here
//!     // E.g.:
//!     let listener = std::net::TcpListener::bind((server_config.bind_addr, server_config.port)).expect("Failed to bind socket");
//! 
//! }
//! ```

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub(crate) mod config;
pub(crate) mod codegen;

use std::io::{self, Read, Write};

#[derive(Debug)]
enum ErrorData {
    Toml(toml::de::Error),
    Config(config::ValidationError),
    Io(io::Error),
}

/// Error that occured during code generation
///
/// It currently only implements Debug, which should
/// be sufficient for now. This may be improved in the future.
#[derive(Debug)]
pub struct Error {
    data: ErrorData,
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

/// Generates the source code for you from provided `toml` configuration.
pub fn generate_source<S: Read, O: Write>(mut source: S, output: O) -> Result<(), Error> {
    let mut data = Vec::new();
    source.read_to_end(&mut data)?;
    let cfg = toml::from_slice::<config::raw::Config>(&data)?;
    let cfg = cfg.validate()?;
    
    codegen::generate_code(&cfg, output).map_err(Into::into)
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

[[param]]
name = "bar"
type = "String"
optional = true

[[param]]
name = "baz"
type = "String"
optional = false

[[switch]]
name = "verbose"

[[switch]]
name = "fast"
default = true
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

    impl ExpectedOutput {
        fn complete(&self) -> String {
            format!(
r#"#[derive(Debug)]
pub enum ValidationError {{
    MissingField(&'static str),
}}

{}
#[derive(Debug)]
pub enum Error {{
    Reading(::std::io::Error),
    ConfigParsing(::toml::de::Error),
    Arguments(ArgParseError),
    Validation(ValidationError),
}}

impl From<::std::io::Error> for Error {{
    fn from(err: ::std::io::Error) -> Self {{
        Error::Reading(err)
    }}
}}

impl From<::toml::de::Error> for Error {{
    fn from(err: ::toml::de::Error) -> Self {{
        Error::ConfigParsing(err)
    }}
}}

impl From<ArgParseError> for Error {{
    fn from(err: ArgParseError) -> Self {{
        Error::Arguments(err)
    }}
}}

impl From<ValidationError> for Error {{
    fn from(err: ValidationError) -> Self {{
        Error::Validation(err)
    }}
}}

mod raw {{
    use ::std::path::PathBuf;
    use super::{{ArgParseError, ValidationError}};
{}
    impl Config {{
        pub fn load<P: AsRef<::std::path::Path>>(config_file: P) -> Result<Self, super::Error> {{
            use std::io::Read;

            let mut config_file = ::std::fs::File::open(config_file)?;
            let mut config_content = Vec::new();
            config_file.read_to_end(&mut config_content)?;
            ::toml::from_slice(&config_content).map_err(Into::into)
        }}

{}
{}
{}    }}
}}

{}
impl Config {{
    pub fn including_optional_config_files<I>(config_files: I) -> Result<Self, Error> where I: IntoIterator, I::Item: AsRef<::std::path::Path> {{
        let mut config = raw::Config::default();
        for path in config_files {{
            match raw::Config::load(path) {{
                Ok(new_config) => config.merge_in(new_config),
                Err(Error::Reading(ref err)) if err.kind() == ::std::io::ErrorKind::NotFound => (),
                Err(err) => return Err(err),
            }}
        }}
        config.merge_args(::std::env::args_os())?;
        config.validate().map_err(Into::into)
    }}
}}
"#,
            self.arg_parse_error,
            self.raw_config,
            self.validate,
            self.merge_in,
            self.merge_args,
            self.config
                   )
        }
    }

    pub const EXPECTED_EMPTY: ExpectedOutput = ExpectedOutput {
        raw_config:
r#"    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
    }
"#,
        validate:
r#"        pub fn validate(self) -> Result<super::Config, ValidationError> {

            Ok(super::Config {
            })
        }
"#,
        merge_in:
r#"        pub fn merge_in(&mut self, other: Self) {
        }
"#,
        merge_args:
r#"        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<(), super::Error> {
            let mut iter = args.into_iter();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    break;
                } else {
                    return Err(ArgParseError::UnknownArgument.into());
                }
            }

            Ok(())
        }
"#,
        config:
r#"/// Configuration of the application
pub struct Config {
}
"#,
        arg_parse_error:
r#"#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

}
"#,
    };

    pub const EXPECTED_SINGLE_OPTIONAL_PARAM: ExpectedOutput = ExpectedOutput {
        raw_config:
r#"    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
        foo: Option<u32>,
    }
"#,
        validate:
r#"        pub fn validate(self) -> Result<super::Config, ValidationError> {
            let foo = self.foo;

            Ok(super::Config {
                foo,
            })
        }
"#,
        merge_in:
r#"        pub fn merge_in(&mut self, other: Self) {
            if self.foo.is_none() {
                self.foo = other.foo;
            }
        }
"#,
        merge_args:
r#"        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<(), super::Error> {
            let mut iter = args.into_iter();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    break;
                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = foo
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--foo"))?
                        .parse()
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
                } else {
                    return Err(ArgParseError::UnknownArgument.into());
                }
            }

            Ok(())
        }
"#,
        config:
r#"/// Configuration of the application
pub struct Config {
    pub foo: Option<u32>,
}
"#,
        arg_parse_error:
r#"#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

    FieldFoo(<u32 as ::std::str::FromStr>::Err),
}
"#,
    };

    pub const EXPECTED_SINGLE_MANDATORY_PARAM: ExpectedOutput = ExpectedOutput {
        raw_config:
r#"    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
        foo: Option<u32>,
    }
"#,
        validate:
r#"        pub fn validate(self) -> Result<super::Config, ValidationError> {
            let foo = self.foo.ok_or(ValidationError::MissingField("foo"))?;

            Ok(super::Config {
                foo,
            })
        }
"#,
        merge_in:
r#"        pub fn merge_in(&mut self, other: Self) {
            if self.foo.is_none() {
                self.foo = other.foo;
            }
        }
"#,
        merge_args:
r#"        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<(), super::Error> {
            let mut iter = args.into_iter();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    break;
                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = foo
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--foo"))?
                        .parse()
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
                } else {
                    return Err(ArgParseError::UnknownArgument.into());
                }
            }

            Ok(())
        }
"#,
        config:
r#"/// Configuration of the application
pub struct Config {
    pub foo: u32,
}
"#,
        arg_parse_error:
r#"#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

    FieldFoo(<u32 as ::std::str::FromStr>::Err),
}
"#,
    };

    pub const EXPECTED_SINGLE_DEFAULT_PARAM: ExpectedOutput = ExpectedOutput {
        raw_config:
r#"    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
        foo: Option<u32>,
    }
"#,
        validate:
r#"        pub fn validate(self) -> Result<super::Config, ValidationError> {
            let foo = self.foo.unwrap_or_else(|| { 42 });

            Ok(super::Config {
                foo,
            })
        }
"#,
        merge_in:
r#"        pub fn merge_in(&mut self, other: Self) {
            if self.foo.is_none() {
                self.foo = other.foo;
            }
        }
"#,
        merge_args:
r#"        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<(), super::Error> {
            let mut iter = args.into_iter();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    break;
                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = foo
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--foo"))?
                        .parse()
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
                } else {
                    return Err(ArgParseError::UnknownArgument.into());
                }
            }

            Ok(())
        }
"#,
        config:
r#"/// Configuration of the application
pub struct Config {
    pub foo: u32,
}
"#,
        arg_parse_error:
r#"#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

    FieldFoo(<u32 as ::std::str::FromStr>::Err),
}
"#,
    };

    pub const EXPECTED_SINGLE_SWITCH: ExpectedOutput = ExpectedOutput {
        raw_config:
r#"    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
        foo: Option<bool>,
    }
"#,
        validate:
r#"        pub fn validate(self) -> Result<super::Config, ValidationError> {

            Ok(super::Config {
                foo: self.foo.unwrap_or(false),
            })
        }
"#,
        merge_in:
r#"        pub fn merge_in(&mut self, other: Self) {
            if self.foo.is_none() {
                self.foo = other.foo;
            }
        }
"#,
        merge_args:
r#"        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<(), super::Error> {
            let mut iter = args.into_iter();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    break;
                } else if arg == *"--foo" {
                    self.foo = Some(true);
                } else {
                    return Err(ArgParseError::UnknownArgument.into());
                }
            }

            Ok(())
        }
"#,
        config:
r#"/// Configuration of the application
pub struct Config {
    pub foo: bool,
}
"#,
        arg_parse_error:
r#"#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

}
"#,
    };

    pub const EXPECTED_MULTIPLE_PARAMS: ExpectedOutput = ExpectedOutput {
        raw_config:
r#"    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
        foo: Option<u32>,
        bar: Option<String>,
        baz: Option<String>,
        verbose: Option<bool>,
        fast: Option<bool>,
    }
"#,
        validate:
r#"        pub fn validate(self) -> Result<super::Config, ValidationError> {
            let foo = self.foo.unwrap_or_else(|| { 42 });
            let bar = self.bar;
            let baz = self.baz.ok_or(ValidationError::MissingField("baz"))?;

            Ok(super::Config {
                foo,
                bar,
                baz,
                verbose: self.verbose.unwrap_or(false),
                fast: self.fast.unwrap_or(true),
            })
        }
"#,
        merge_in:
r#"        pub fn merge_in(&mut self, other: Self) {
            if self.foo.is_none() {
                self.foo = other.foo;
            }
            if self.bar.is_none() {
                self.bar = other.bar;
            }
            if self.baz.is_none() {
                self.baz = other.baz;
            }
            if self.verbose.is_none() {
                self.verbose = other.verbose;
            }
            if self.fast.is_none() {
                self.fast = other.fast;
            }
        }
"#,
        merge_args:
r#"        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<(), super::Error> {
            let mut iter = args.into_iter();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    break;
                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = foo
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--foo"))?
                        .parse()
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
                } else if arg == *"--bar" {
                    let bar = iter.next().ok_or(ArgParseError::MissingArgument("--bar"))?;

                    let bar = bar
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--bar"))?
                        .parse()
                        .map_err(ArgParseError::FieldBar)?;

                    self.bar = Some(bar);
                } else if arg == *"--baz" {
                    let baz = iter.next().ok_or(ArgParseError::MissingArgument("--baz"))?;

                    let baz = baz
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--baz"))?
                        .parse()
                        .map_err(ArgParseError::FieldBaz)?;

                    self.baz = Some(baz);
                } else if arg == *"--verbose" {
                    self.verbose = Some(true);
                } else if arg == *"--no-fast" {
                    self.fast = Some(false);
                } else {
                    return Err(ArgParseError::UnknownArgument.into());
                }
            }

            Ok(())
        }
"#,
        config:
r#"/// Configuration of the application
pub struct Config {
    pub foo: u32,
    pub bar: Option<String>,
    pub baz: String,
    pub verbose: bool,
    pub fast: bool,
}
"#,
        arg_parse_error:
r#"#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

    FieldFoo(<u32 as ::std::str::FromStr>::Err),
    FieldBar(<String as ::std::str::FromStr>::Err),
    FieldBaz(<String as ::std::str::FromStr>::Err),
}
"#,
    };

    pub const EXPECTED_NO_ARG: ExpectedOutput = ExpectedOutput {
        raw_config:
r#"    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
        foo: Option<u32>,
    }
"#,
        validate:
r#"        pub fn validate(self) -> Result<super::Config, ValidationError> {
            let foo = self.foo;

            Ok(super::Config {
                foo,
            })
        }
"#,
        merge_in:
r#"        pub fn merge_in(&mut self, other: Self) {
            if self.foo.is_none() {
                self.foo = other.foo;
            }
        }
"#,
        merge_args:
r#"        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<(), super::Error> {
            let mut iter = args.into_iter();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    break;
                } else {
                    return Err(ArgParseError::UnknownArgument.into());
                }
            }

            Ok(())
        }
"#,
        config:
r#"/// Configuration of the application
pub struct Config {
    pub foo: Option<u32>,
}
"#,
        arg_parse_error:
r#"#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

}
"#,
    };


    fn check(src: &str, expected: &str) {
        let mut src = src.as_bytes();
        let mut out = Vec::new();
        generate_source(&mut src, &mut out).unwrap();
        assert_eq!(::std::str::from_utf8(&out).unwrap(), expected);
    }

    #[test]
    fn empty() {
        check("", &EXPECTED_EMPTY.complete());
    }

    #[test]
    fn single_optional_param() {
        check(SINGLE_OPTIONAL_PARAM, &EXPECTED_SINGLE_OPTIONAL_PARAM.complete());
    }

    #[test]
    fn single_mandatory_param() {
        check(SINGLE_MANDATORY_PARAM, &EXPECTED_SINGLE_MANDATORY_PARAM.complete());
    }

    #[test]
    fn single_default_param() {
        check(SINGLE_DEFAULT_PARAM, &EXPECTED_SINGLE_DEFAULT_PARAM.complete());
    }

    #[test]
    fn single_switch() {
        check(SINGLE_SWITCH, &EXPECTED_SINGLE_SWITCH.complete());
    }

    #[test]
    fn multiple_params() {
        check(MULTIPLE_PARAMS, &EXPECTED_MULTIPLE_PARAMS.complete());
    }

    #[test]
    fn no_arg() {
        check(NO_ARG, &EXPECTED_NO_ARG.complete());
    }
}
