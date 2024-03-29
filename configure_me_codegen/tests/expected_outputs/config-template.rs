pub mod prelude {
    pub use super::{Config, ResultExt};
}

pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument(String),
    HelpRequested(String),

<<"arg_parse_error.rs">>
}

#[automatically_derived]
impl ::std::fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ArgParseError::MissingArgument(arg) => write!(f, "A value to argument '{}' is missing.", arg),
            ArgParseError::UnknownArgument(arg) => write!(f, "An unknown argument '{}' was specified.", arg),
<<"display_arg_parse_error.rs">>
        }
    }
}

#[automatically_derived]
impl ::std::fmt::Debug for ArgParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(self, f)
    }
}

pub enum EnvParseError {
<<"env_parse_error.rs">>
}

#[automatically_derived]
impl ::std::fmt::Display for EnvParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
<<"display_env_parse_error.rs">>
        }
    }
}

#[automatically_derived]
impl ::std::fmt::Debug for EnvParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(self, f)
    }
}

pub enum ValidationError {
<<"validation_error.rs">>
}

#[automatically_derived]
impl ::std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
<<"display_validation_error.rs">>
        }
    }
}

#[automatically_derived]
impl ::std::fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(self, f)
    }
}

pub enum Error {
    Reading { file: ::std::path::PathBuf, error: ::std::io::Error },
    ConfigParsing { file: ::std::path::PathBuf, error: ::configure_me::toml::de::Error },
    Arguments(ArgParseError),
    Environment(EnvParseError),
    Validation(ValidationError),
}

#[automatically_derived]
impl From<ArgParseError> for Error {
    fn from(err: ArgParseError) -> Self {
        Error::Arguments(err)
    }
}

#[automatically_derived]
impl From<EnvParseError> for Error {
    fn from(err: EnvParseError) -> Self {
        Error::Environment(err)
    }
}

#[automatically_derived]
impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Error::Validation(err)
    }
}

#[automatically_derived]
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Error::Reading { file, error } => write!(f, "Failed to read configuration file {}: {}", file.display(), error),
            Error::ConfigParsing { file, error } => write!(f, "Failed to parse configuration file {}: {}", file.display(), error),
            Error::Arguments(err) => write!(f, "{}", err),
            Error::Environment(err) => write!(f, "{}", err),
            Error::Validation(err) => write!(f, "Invalid configuration: {}", err),
        }
    }
}

#[automatically_derived]
impl ::std::fmt::Debug for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(self, f)
    }
}

mod raw {
    use super::{ArgParseError, ValidationError};

    #[derive(Deserialize, Default)]
    #[serde(crate = "crate::configure_me::serde")]
    pub struct Config {
<<"raw_config.rs">>
    }

    #[automatically_derived]
    impl Config {
        pub fn load<P: AsRef<::std::path::Path>>(config_file_name: P) -> Result<Self, super::Error> {
            use std::io::Read;

            let mut config_file = ::std::fs::File::open(&config_file_name).map_err(|error| super::Error::Reading { file: config_file_name.as_ref().into(), error })?;
            let mut config_content = Vec::new();
            config_file.read_to_end(&mut config_content).map_err(|error| super::Error::Reading { file: config_file_name.as_ref().into(), error })?;
            ::configure_me::toml::from_slice(&config_content).map_err(|error| super::Error::ConfigParsing { file: config_file_name.as_ref().into(), error })
        }

        pub fn validate(self) -> Result<super::Config, ValidationError> {
<<"validate.rs">>
        }

        pub fn merge_in(&mut self, other: Self) {
<<"merge_in.rs">>
        }

        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I, skip_default_conf_files: &mut bool) -> Result<(Option<std::path::PathBuf>, impl Iterator<Item=::std::ffi::OsString>), super::Error> {
            let _ = skip_default_conf_files;
            let mut iter = args.into_iter().fuse();
            let program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    return Ok((program_path, None.into_iter().chain(iter)));
                } else if (arg == *"--help") || (arg == *"-h") {
                    return Err(ArgParseError::HelpRequested(program_path.as_ref().unwrap().to_string_lossy().into()).into());
<<"merge_args.rs">>
                } else if let Some(mut shorts) = ::configure_me::parse_arg::iter_short(&arg) {
                    for short in &mut shorts {
                        if short == 'h' {
                            return Err(ArgParseError::HelpRequested(program_path.as_ref().unwrap().to_string_lossy().into()).into())
<<"merge_short_args.rs">>
                        } else {
                            let mut arg = String::with_capacity(2);
                            arg.push('-');
                            arg.push(short);
                            return Err(ArgParseError::UnknownArgument(arg).into());
                        }
                    }
                } else if arg.to_str().unwrap_or("").starts_with("--") {
                    return Err(ArgParseError::UnknownArgument(arg.into_string().unwrap()).into());
                } else {
                    return Ok((program_path, Some(arg).into_iter().chain(iter)))
                }
            }

            Ok((program_path, None.into_iter().chain(iter)))
        }

        pub fn merge_env(&mut self) -> Result<(), super::Error> {
<<"merge_env.rs">>
            Ok(())
        }
    }
}

/// Configuration of the application
pub struct Config {
<<"config.rs">>
}

#[automatically_derived]
impl Config {
    pub fn including_optional_config_files<I>(config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>, Metadata), Error> where I: IntoIterator, I::Item: AsRef<::std::path::Path> {
        Self::custom_args_and_optional_files(::std::env::args_os(), config_files)
    }

    pub fn custom_args_and_optional_files<A, I>(args: A, config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>, Metadata), Error> where
        A: IntoIterator, A::Item: Into<::std::ffi::OsString>,
        I: IntoIterator, I::Item: AsRef<::std::path::Path> {

        let mut args_config = raw::Config::default();
        let mut skip_default_conf_files = false;
        let (program_name, remaining_args) = args_config.merge_args(args.into_iter().map(Into::into), &mut skip_default_conf_files)?;
<<"process_program_name.rs">>

        let mut config = raw::Config::default();

        if !skip_default_conf_files {
            for path in config_files {
                match raw::Config::load(path) {
                    Ok(mut new_config) => {
                        std::mem::swap(&mut config, &mut new_config);
                        config.merge_in(new_config)
                    },
                    Err(Error::Reading { ref error, .. }) if error.kind() == ::std::io::ErrorKind::NotFound => (),
                    Err(err) => return Err(err),
                }
            }
        }

        config.merge_env()?;
        config.merge_in(args_config);

        let metadata = Metadata {
<<"construct_metadata.rs">>
        };

        config
            .validate()
            .map(|cfg| (cfg, remaining_args, metadata))
            .map_err(Into::into)
    }
}

/// Metadata of the configuration.
///
/// This struct provides some additional information regarding the configuration.
/// Currently it only contains program name but more items could be available in the future.
#[non_exhaustive]
pub struct Metadata {
<<"metadata_fields.rs">>
}

pub trait ResultExt {
    type Item;

    fn unwrap_or_exit(self) -> Self::Item;
}

#[automatically_derived]
impl<T> ResultExt for Result<T, Error> {
    type Item = T;

    fn unwrap_or_exit(self) -> Self::Item {
        use std::io::Write;

        match self {
            Ok(item) => item,
            Err(err @ Error::Arguments(ArgParseError::HelpRequested(_))) => {
                println!("{}", err);
                std::io::stdout().flush().expect("failed to flush stdout");
                ::std::process::exit(0)
            },
            Err(err) => {
                eprintln!("Error: {}", err);
                std::io::stderr().flush().expect("failed to flush stderr");
                ::std::process::exit(1)
            }
        }
    }
}
