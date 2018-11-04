use std::io::{self, Write};
use ::config::{Config, Optionality, SwitchKind};

fn gen_raw_params<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        writeln!(output, "        {}: Option<{}>,", param.name, param.ty)?;
    }
    Ok(())
}

fn gen_raw_switches<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for switch in &config.switches {
        writeln!(output, "        {}: Option<bool>,", switch.name)?;
    }
    Ok(())
}

fn gen_raw_config<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    gen_raw_params(config, &mut output)?;
    gen_raw_switches(config, &mut output)?;
    Ok(())
}

fn gen_arg_parse_error<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        if !param.argument {
            continue;
        }

        write!(output, "    Field")?;
        pascal_case(&mut output, &param.name)?;
        writeln!(output, "(<{} as ::std::str::FromStr>::Err),", param.ty)?;
    }
    Ok(())
}

fn gen_display_arg_parse_error<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        if !param.argument {
            continue;
        }

        write!(output, "        ArgParseError::Field")?;
        pascal_case(&mut output, &param.name)?;
        write!(output, "(err) => write!(f, \"Failed to parse argument '--")?;
        underscore_to_hypen(&mut output, &param.name)?;
        writeln!(output, "': {{}}.\", err),")?;
    }
    Ok(())
}

fn gen_params<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        match param.optionality {
            Optionality::Optional => writeln!(output, "    pub {}: Option<{}>,", param.name, param.ty)?,
            _ => writeln!(output, "    pub {}: {},", param.name, param.ty)?,
        }
    }
    Ok(())
}

fn gen_switches<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for switch in &config.switches {
        writeln!(output, "    pub {}: bool,", switch.name)?;
    }
    Ok(())
}

fn gen_param_validation<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        match param.optionality {
            Optionality::Optional => writeln!(output, "            let {} = self.{};", param.name, param.name)?,
            Optionality::Mandatory => writeln!(output, "            let {} = self.{}.ok_or(ValidationError::MissingField(\"{}\"))?;", param.name, param.name, param.name)?,
            Optionality::DefaultValue(ref val) => writeln!(output, "            let {} = self.{}.unwrap_or_else(|| {{ {} }});", param.name, param.name, val)?,
        }
    }
    Ok(())
}

fn gen_construct_config_params<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        writeln!(output, "                {},", param.name)?;
    }
    Ok(())
}

fn gen_copy_switches<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for switch in &config.switches {
        let default_value = match switch.kind {
            SwitchKind::Inverted => "true",
            _ => "false",
        };
        writeln!(output, "                {}: self.{}.unwrap_or({}),", switch.name, switch.name, default_value)?;
    }
    Ok(())
}

fn gen_validation_fn<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    gen_param_validation(config, &mut output)?;
    writeln!(output)?;
    writeln!(output, "            Ok(super::Config {{")?;
    gen_construct_config_params(config, &mut output)?;
    gen_copy_switches(config, &mut output)?;
    writeln!(output, "            }})")?;
    Ok(())
}

fn gen_merge_in<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        writeln!(output, "            if self.{}.is_none() {{", param.name)?;
        writeln!(output, "                self.{} = other.{};", param.name, param.name)?;
        writeln!(output, "            }}")?;
    }
    for switch in &config.switches {
        writeln!(output, "            if self.{}.is_none() {{", switch.name)?;
        writeln!(output, "                self.{} = other.{};", switch.name, switch.name)?;
        writeln!(output, "            }}")?;
    }
    Ok(())
}

fn pascal_case<W: Write>(mut output: W, ident: &str) -> io::Result<()> {
    let mut next_big = true;
    for c in ident.chars() {
        match (c, next_big) {
            ('_', _) => next_big = true,
            (x, true) => {
                write!(output, "{}", x.to_ascii_uppercase())?;
                next_big = false;
            },
            (x, false) => write!(output, "{}", x)?,
        }
    }
    Ok(())
}

fn underscore_to_hypen<W: Write>(mut output: W, ident: &str) -> io::Result<()> {
    for c in ident.chars() {
        if c == '_' {
                write!(output, "-")?;
        } else {
                write!(output, "{}", c)?;
        }
    }
    Ok(())
}

fn gen_arg_parse_params<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for param in &config.params {
        if !param.argument {
            continue;
        }

        write!(output, "                }} else if arg == *\"--")?;
        underscore_to_hypen(&mut output, &param.name)?;
        writeln!(output, "\" {{")?;
        write!(output, "                    let {} = iter.next().ok_or(ArgParseError::MissingArgument(\"--", &param.name)?;
        underscore_to_hypen(&mut output, &param.name)?;
        writeln!(output, "\"))?;")?;
        writeln!(output)?;
        writeln!(output, "                    let {} = {}", param.name, param.name)?;
        writeln!(output, "                        .to_str()")?;
        write!(output, "                        .ok_or(ArgParseError::BadUtf8(\"--")?;
        underscore_to_hypen(&mut output, &param.name)?;
        writeln!(output, "\"))?")?;
        writeln!(output, "                        .parse()")?;
        write!(  output, "                        .map_err(ArgParseError::Field")?;
        pascal_case(&mut output, &param.name)?;
        writeln!(output, ")?;")?;
        writeln!(output)?;
        writeln!(output, "                    self.{} = Some({});", param.name, param.name)?;
    }
    Ok(())
}

fn gen_merge_args<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    gen_arg_parse_params(config, &mut output)?;
    gen_arg_parse_switches(config, &mut output)?;
    Ok(())
}

fn gen_arg_parse_switches<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    for switch in &config.switches {
        match switch.kind {
            SwitchKind::Inverted => {
                writeln!(output, "                }} else if arg == *\"--no-{}\" {{", switch.name)?;
                writeln!(output, "                    self.{} = Some(false);", switch.name)?;
            },
            _ => {
                writeln!(output, "                }} else if arg == *\"--{}\" {{", switch.name)?;
                writeln!(output, "                    self.{} = Some(true);", switch.name)?;
            }
        }
    }
    Ok(())
}

pub fn generate_code<W: Write>(config: &Config, mut output: W) -> io::Result<()> {
    writeln!(output, "pub enum ArgParseError {{")?;
    writeln!(output, "    MissingArgument(&'static str),")?;
    writeln!(output, "    UnknownArgument(String),")?;
    writeln!(output, "    BadUtf8(&'static str),")?;
    writeln!(output)?;
    gen_arg_parse_error(config, &mut output)?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Display for ArgParseError {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        match self {{")?;
    writeln!(output, "            ArgParseError::MissingArgument(arg) => write!(f, \"A value to argument '{{}}' is missing.\", arg),")?;
    writeln!(output, "            ArgParseError::UnknownArgument(arg) => write!(f, \"An unknown argument '{{}}' was specified.\", arg),")?;
    writeln!(output, "            ArgParseError::BadUtf8(arg) => write!(f, \"The argument '{{}}' doesn't have valid UTF-8 encoding.\", arg),")?;
    gen_display_arg_parse_error(config, &mut output)?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Debug for ArgParseError {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        ::std::fmt::Display::fmt(self, f)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "pub enum ValidationError {{")?;
    writeln!(output, "    MissingField(&'static str),")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Display for ValidationError {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        match self {{")?;
    writeln!(output, "            ValidationError::MissingField(field) => write!(f, \"Configuration parameter '{{}}' not specified.\", field),")?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Debug for ValidationError {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        ::std::fmt::Display::fmt(self, f)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "pub enum Error {{")?;
    writeln!(output, "    Reading {{ file: ::std::path::PathBuf, error: ::std::io::Error }},")?;
    writeln!(output, "    ConfigParsing {{ file: ::std::path::PathBuf, error: ::toml::de::Error }},")?;
    writeln!(output, "    Arguments(ArgParseError),")?;
    writeln!(output, "    Validation(ValidationError),")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl From<ArgParseError> for Error {{")?;
    writeln!(output, "    fn from(err: ArgParseError) -> Self {{")?;
    writeln!(output, "        Error::Arguments(err)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl From<ValidationError> for Error {{")?;
    writeln!(output, "    fn from(err: ValidationError) -> Self {{")?;
    writeln!(output, "        Error::Validation(err)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Display for Error {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        match self {{")?;
    writeln!(output, "            Error::Reading {{ file, error }} => write!(f, \"Failed to read configuration file {{}}: {{}}\", file.display(), error),")?;
    writeln!(output, "            Error::ConfigParsing {{ file, error }} => write!(f, \"Failed to parse configuration file {{}}: {{}}\", file.display(), error),")?;
    writeln!(output, "            Error::Arguments(err) => write!(f, \"{{}}\", err),")?;
    writeln!(output, "            Error::Validation(err) => write!(f, \"Invalid configuration: {{}}\", err),")?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Debug for Error {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        ::std::fmt::Display::fmt(self, f)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "mod raw {{")?;
    writeln!(output, "    use ::std::path::PathBuf;")?;
    writeln!(output, "    use super::{{ArgParseError, ValidationError}};")?;
    writeln!(output)?;
    writeln!(output, "    #[derive(Deserialize, Default)]")?;
    writeln!(output, "    pub struct Config {{")?;
    writeln!(output, "        _program_path: Option<PathBuf>,")?;
    gen_raw_config(config, &mut output)?;
    writeln!(output, "    }}")?;
    writeln!(output)?;
    writeln!(output, "    impl Config {{")?;
    writeln!(output, "        pub fn load<P: AsRef<::std::path::Path>>(config_file_name: P) -> Result<Self, super::Error> {{")?;
    writeln!(output, "            use std::io::Read;")?;
    writeln!(output)?;
    writeln!(output, "            let mut config_file = ::std::fs::File::open(&config_file_name).map_err(|error| super::Error::Reading {{ file: config_file_name.as_ref().into(), error }})?;")?;
    writeln!(output, "            let mut config_content = Vec::new();")?;
    writeln!(output, "            config_file.read_to_end(&mut config_content).map_err(|error| super::Error::Reading {{ file: config_file_name.as_ref().into(), error }})?;")?;
    writeln!(output, "            ::toml::from_slice(&config_content).map_err(|error| super::Error::ConfigParsing {{ file: config_file_name.as_ref().into(), error }})")?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        pub fn validate(self) -> Result<super::Config, ValidationError> {{")?;
    gen_validation_fn(config, &mut output)?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        pub fn merge_in(&mut self, other: Self) {{")?;
    gen_merge_in(config, &mut output)?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<impl Iterator<Item=::std::ffi::OsString>, super::Error> {{")?;
    writeln!(output, "            let mut iter = args.into_iter().fuse();")?;
    writeln!(output, "            self._program_path = iter.next().map(Into::into);")?;
    writeln!(output)?;
    writeln!(output, "            while let Some(arg) = iter.next() {{")?;
    writeln!(output, "                if arg == *\"--\" {{")?;
    writeln!(output, "                    return Ok(None.into_iter().chain(iter));")?;
    gen_merge_args(config, &mut output)?;
    writeln!(output, "                }} else if arg.to_str().unwrap_or(\"\").starts_with(\"--\") {{")?;
    writeln!(output, "                    return Err(ArgParseError::UnknownArgument(arg.into_string().unwrap()).into());")?;
    writeln!(output, "                }} else {{")?;
    writeln!(output, "                    return Ok(Some(arg).into_iter().chain(iter))")?;
    writeln!(output, "                }}")?;
    writeln!(output, "            }}")?;
    writeln!(output)?;
    writeln!(output, "            Ok(None.into_iter().chain(iter))")?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "/// Configuration of the application")?;
    writeln!(output, "pub struct Config {{")?;
    gen_params(config, &mut output)?;
    gen_switches(config, &mut output)?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl Config {{")?;
    writeln!(output, "    pub fn including_optional_config_files<I>(config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>), Error> where I: IntoIterator, I::Item: AsRef<::std::path::Path> {{")?;
    writeln!(output, "        Config::custom_args_and_optional_files(::std::env::args_os(), config_files)")?;
    writeln!(output, "    }}")?;
    writeln!(output)?;
    writeln!(output, "    pub fn custom_args_and_optional_files<A, I>(args: A, config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>), Error> where")?;
    writeln!(output, "        A: IntoIterator, A::Item: Into<::std::ffi::OsString>,")?;
    writeln!(output, "        I: IntoIterator, I::Item: AsRef<::std::path::Path> {{")?;
    writeln!(output)?;
    writeln!(output, "        let mut config = raw::Config::default();")?;
    writeln!(output, "        for path in config_files {{")?;
    writeln!(output, "            match raw::Config::load(path) {{")?;
    writeln!(output, "                Ok(new_config) => config.merge_in(new_config),")?;
    writeln!(output, "                Err(Error::Reading {{ ref error, .. }}) if error.kind() == ::std::io::ErrorKind::NotFound => (),")?;
    writeln!(output, "                Err(err) => return Err(err),")?;
    writeln!(output, "            }}")?;
    writeln!(output, "        }}")?;
    writeln!(output, "        let remaining_args = config.merge_args(args.into_iter().map(Into::into))?;")?;
    writeln!(output, "        config")?;
    writeln!(output, "            .validate()")?;
    writeln!(output, "            .map(|cfg| (cfg, remaining_args))")?;
    writeln!(output, "            .map_err(Into::into)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ::config::Config;

    fn config_from(input: &str) -> Config {
        ::toml::from_str::<::config::raw::Config>(input).unwrap().validate().unwrap()
    }

    fn config_empty() -> Config {
        ::toml::from_str::<::config::raw::Config>("").unwrap().validate().unwrap()
    }

    macro_rules! check {
        ($fn:ident, $config:expr, $expected:expr) => {
            let mut out = Vec::new();
            super::$fn($config, &mut out).unwrap();
            assert_eq!(::std::str::from_utf8(&out).unwrap(), $expected);
        }
    }

    #[test]
    fn empty_raw_config() {
        check!(gen_raw_config, &config_from(""), ::tests::EXPECTED_EMPTY.raw_config);
    }

    #[test]
    fn single_optional_param_raw_config() {
        check!(gen_raw_config, &config_from(::tests::SINGLE_OPTIONAL_PARAM), ::tests::EXPECTED_SINGLE_OPTIONAL_PARAM.raw_config);
    }

    #[test]
    fn single_optional_param_validate() {
        check!(gen_validation_fn, &config_from(::tests::SINGLE_OPTIONAL_PARAM), ::tests::EXPECTED_SINGLE_OPTIONAL_PARAM.validate);
    }

    #[test]
    fn single_optional_merge_args() {
        check!(gen_merge_args, &config_from(::tests::SINGLE_OPTIONAL_PARAM), ::tests::EXPECTED_SINGLE_OPTIONAL_PARAM.merge_args);
    }

    #[test]
    fn single_mandatory_param_raw_config() {
        check!(gen_raw_config, &config_from(::tests::SINGLE_MANDATORY_PARAM), ::tests::EXPECTED_SINGLE_MANDATORY_PARAM.raw_config);
    }

    #[test]
    fn single_default_param_raw_config() {
        check!(gen_raw_config, &config_from(::tests::SINGLE_DEFAULT_PARAM), ::tests::EXPECTED_SINGLE_DEFAULT_PARAM.raw_config);
    }

    #[test]
    fn single_switch_raw_config() {
        check!(gen_raw_config, &config_from(::tests::SINGLE_SWITCH), ::tests::EXPECTED_SINGLE_SWITCH.raw_config);
    }

    #[test]
    fn empty_arg_parse_error() {
        let expected = "";
        check!(gen_arg_parse_error, &config_empty(), expected);
    }

}
