#[derive(Debug)]
pub enum ArgParseError {
    MissingArgument(&'static str),
    UnknownArgument,
    BadUtf8(&'static str),

<<"arg_parse_error.rs">>
}

#[derive(Debug)]
pub enum ValidationError {
    MissingField(&'static str),
}

#[derive(Debug)]
pub enum Error {
    Reading(::std::io::Error),
    ConfigParsing(::toml::de::Error),
    Arguments(ArgParseError),
    Validation(ValidationError),
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Self {
        Error::Reading(err)
    }
}

impl From<::toml::de::Error> for Error {
    fn from(err: ::toml::de::Error) -> Self {
        Error::ConfigParsing(err)
    }
}

impl From<ArgParseError> for Error {
    fn from(err: ArgParseError) -> Self {
        Error::Arguments(err)
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Error::Validation(err)
    }
}

mod raw {
    use ::std::path::PathBuf;
    use super::{ArgParseError, ValidationError};

    #[derive(Deserialize, Default)]
    pub struct Config {
        _program_path: Option<PathBuf>,
<<"raw_config.rs">>
    }

    impl Config {
        pub fn load<P: AsRef<::std::path::Path>>(config_file: P) -> Result<Self, super::Error> {
            use std::io::Read;

            let mut config_file = ::std::fs::File::open(config_file)?;
            let mut config_content = Vec::new();
            config_file.read_to_end(&mut config_content)?;
            ::toml::from_slice(&config_content).map_err(Into::into)
        }

        pub fn validate(self) -> Result<super::Config, ValidationError> {
<<"validate.rs">>
        }

        pub fn merge_in(&mut self, other: Self) {
<<"merge_in.rs">>
        }

        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<impl Iterator<Item=::std::ffi::OsString>, super::Error> {
            let mut iter = args.into_iter().fuse();
            self._program_path = iter.next().map(Into::into);

            while let Some(arg) = iter.next() {
                if arg == *"--" {
                    return Ok(None.into_iter().chain(iter));
<<"merge_args.rs">>
                } else if arg.to_str().unwrap_or("").starts_with("--") {
                    return Err(ArgParseError::UnknownArgument.into());
                } else {
                    return Ok(Some(arg).into_iter().chain(iter))
                }
            }

            Ok(None.into_iter().chain(iter))
        }
    }
}

/// Configuration of the application
pub struct Config {
<<"config.rs">>
}

impl Config {
    pub fn including_optional_config_files<I>(config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>), Error> where I: IntoIterator, I::Item: AsRef<::std::path::Path> {
        Config::custom_args_and_optional_files(::std::env::args_os(), config_files)
    }

    pub fn custom_args_and_optional_files<A, I>(args: A, config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>), Error> where
        A: IntoIterator, A::Item: Into<::std::ffi::OsString>,
        I: IntoIterator, I::Item: AsRef<::std::path::Path> {

        let mut config = raw::Config::default();
        for path in config_files {
            match raw::Config::load(path) {
                Ok(new_config) => config.merge_in(new_config),
                Err(Error::Reading(ref err)) if err.kind() == ::std::io::ErrorKind::NotFound => (),
                Err(err) => return Err(err),
            }
        }
        let remaining_args = config.merge_args(args.into_iter().map(Into::into))?;
        config
            .validate()
            .map(|cfg| (cfg, remaining_args))
            .map_err(Into::into)
    }
}
