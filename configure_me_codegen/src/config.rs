use std::fmt;

#[derive(Debug)]
pub enum ValidationErrorKind {
    MandatoryWithDefault,
    InvertedWithAbbr,
    InvertedWithCount,
    InvalidAbbr,
}

#[derive(Debug)]
pub struct ValidationError {
    name: String,
    kind: ValidationErrorKind,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ValidationErrorKind::*;

        let msg = match self.kind {
            MandatoryWithDefault => "parameter with default value must be optional",
            InvertedWithAbbr => "inverted switch can't have short option",
            InvertedWithCount => "inverted switch can't be count",
            InvalidAbbr => "invalid short switch: must be [a-zA-Z]",
        };

        write!(f, "invalid configuration for field {}: {}", self.name, msg)
    }
}

mod ident {
    use std::convert::TryFrom;
    use std::fmt::{self, Write};

    pub struct Error {
        string: String,
        position: usize,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "\"{}\" is not a valid identifier, invalid char at position {}", self.string, self.position)
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(try_from = "String")]
    pub struct Ident(String);

    impl TryFrom<String> for Ident {
        type Error = Error;

        fn try_from(string: String) -> Result<Ident, Error> {
            let bad_char = string
                .chars()
                .enumerate()
                .find(|&(i, c)| c != '_' && ! ((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9' && i > 0)));

            match bad_char {
                Some((i, _)) => {
                    Err(Error {
                        string,
                        position: i,
                    })
                },
                None => Ok(Ident(string)),
            }
        }
    }

    impl Ident {
        pub(crate) fn as_snake_case(&self) -> &str {
            &self.0
        }

        pub(crate) fn as_upper_case(&self) -> UpperCase<'_> {
            UpperCase(&self.0)
        }

        pub(crate) fn as_hypenated(&self) -> Hypenated<'_> {
            Hypenated(&self.0)
        }

        pub(crate) fn as_pascal_case(&self) -> PascalCase<'_> {
            PascalCase(&self.0)
        }
    }

    pub(crate) struct UpperCase<'a>(&'a str);

    impl<'a> fmt::Display for UpperCase<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for ch in self.0.chars() {
                f.write_char(ch.to_ascii_uppercase())?;
            }
            Ok(())
        }
    }

    pub(crate) struct Hypenated<'a>(&'a str);

    impl<'a> fmt::Display for Hypenated<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for ch in self.0.chars() {
                if ch == '_' {
                    f.write_char('-')
                } else {
                    f.write_char(ch)
                }?;
            }
            Ok(())
        }
    }

    pub(crate) struct PascalCase<'a>(&'a str);

    impl<'a> fmt::Display for PascalCase<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut next_big = true;
            for ch in self.0.chars() {
                match (ch, next_big) {
                    ('_', _) => next_big = true,
                    (x, true) => {
                        f.write_char(x.to_ascii_uppercase())?;
                        next_big = false;
                    },
                    (x, false) => f.write_char(x)?,
                }
            }
            Ok(())
        }
    }
}

use self::ident::Ident;

pub mod raw {
    use super::{ValidationError, ValidationErrorKind, Optionality, SwitchKind};
    use super::ident::Ident;

    trait ResultExt {
        type Item;

        fn field_name(self, name: &Ident) -> Result<Self::Item, ValidationError>;
    }

    impl<T> ResultExt for Result<T, ValidationErrorKind> {
        type Item = T;

        fn field_name(self, name: &Ident) -> Result<Self::Item, ValidationError> {
            self.map_err(|kind| ValidationError { name: name.as_snake_case().to_owned(), kind })
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Config {
        #[serde(rename = "param")]
        #[serde(default)]
        pub params: Vec<Param>,
        #[serde(rename = "switch")]
        #[serde(default)]
        pub switches: Vec<Switch>,
        #[serde(default)]
        general: super::General,
        #[serde(default)]
        defaults: super::Defaults,
        #[cfg(feature = "debconf")]
        debconf: Option<::debconf::DebConfig>,
    }

    impl Config {
        pub fn validate(self) -> Result<super::Config, ValidationError> {
            let default_optional = self.defaults.optional;
            let default_argument = self.defaults.args;
            let default_env_var = self.defaults.env_vars.unwrap_or(self.general.env_prefix.is_some());
            let params = self.params
                .into_iter()
                .map(|param| param.validate(default_optional, default_argument, default_env_var))
                .collect::<Result<Vec<_>, _>>()?;

            let switches = self.switches
                .into_iter()
                .map(|switch| switch.validate(default_env_var))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(super::Config {
                general: self.general,
                defaults: self.defaults,
                params,
                switches,
                #[cfg(feature = "debconf")]
                debconf: self.debconf,
            })
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Param {
        name: Ident,
        abbr: Option<char>,
        #[serde(rename = "type")]
        ty: String,
        optional: Option<bool>,
        default: Option<String>,
        doc: Option<String>,
        argument: Option<bool>,
        env_var: Option<bool>,
        convert_into: Option<String>,
        merge_fn: Option<String>,
        #[cfg(feature = "debconf")]
        debconf_priority: Option<::debconf::Priority>,
        #[cfg(feature = "debconf")]
        debconf_default: Option<String>,
    }

    impl Param {
        fn validate_optionality(optional: Option<bool>, default_optional: bool, default: Option<String>) -> Result<Optionality, ValidationErrorKind> {
            match (optional, default_optional, default) {
                (Some(false), _, None) => Ok(Optionality::Mandatory),
                (Some(false), _, Some(_)) => Err(ValidationErrorKind::MandatoryWithDefault),
                (Some(true), _, None) => Ok(Optionality::Optional),
                (_, _, Some(default)) => Ok(Optionality::DefaultValue(default)),
                (None, true, None) => Ok(Optionality::Optional),
                (None, false, None) => Ok(Optionality::Mandatory),
            }
        }

        fn validate(self, default_optional: bool, default_argument: bool, default_env_var: bool) -> Result<super::Param, ValidationError> {
            let optionality = Param::validate_optionality(self.optional, default_optional, self.default)
                .field_name(&self.name)?;

            let ty = self.ty;
            let argument = self.argument.unwrap_or(default_argument);
            let env_var = self.env_var.unwrap_or(default_env_var);
            let convert_into = self.convert_into.unwrap_or_else(|| ty.clone());

            Ok(super::Param {
                name: self.name,
                ty,
                optionality,
                abbr: self.abbr,
                doc: self.doc,
                argument,
                env_var,
                convert_into,
                merge_fn: self.merge_fn,
                #[cfg(feature = "debconf")]
                debconf_priority: self.debconf_priority,
                #[cfg(feature = "debconf")]
                debconf_default: self.debconf_default,
            })
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Switch {
        name: Ident,
        abbr: Option<char>,
        #[serde(default)]
        default: bool,
        doc: Option<String>,
        env_var: Option<bool>,
        #[serde(default)]
        count: bool,
        #[cfg(feature = "debconf")]
        debconf_priority: Option<::debconf::Priority>,
    }

    impl Switch {
        fn validate_abbr(abbr: char) -> Result<char, ValidationErrorKind> {
            if (abbr >= 'a' && abbr <= 'z') || (abbr >= 'A' && abbr <= 'Z') {
                Ok(abbr)
            } else {
                Err(ValidationErrorKind::InvalidAbbr)
            }
        }

        fn validate_kind(abbr: Option<char>, default: bool, count: bool) -> Result<SwitchKind, ValidationErrorKind> {
            match (abbr, default, count) {
                (Some(_), true, _) => Err(ValidationErrorKind::InvertedWithAbbr),
                (_, true, true) => Err(ValidationErrorKind::InvertedWithCount),
                (None, true, false) => Ok(SwitchKind::Inverted),
                (abbr, false, count) => Ok(SwitchKind::Normal { abbr, count }),
            }
        }

        fn validate(self, default_env_var: bool) -> Result<super::Switch, ValidationError> {
            let abbr = self.abbr
                .map(Switch::validate_abbr)
                .transpose()
                .field_name(&self.name)?;

            let kind = Switch::validate_kind(abbr, self.default, self.count)
                .field_name(&self.name)?;

            Ok(super::Switch {
                name: self.name,
                kind,
                doc: self.doc,
                env_var: self.env_var.unwrap_or(default_env_var),
                #[cfg(feature = "debconf")]
                debconf_priority: self.debconf_priority,
            })
        }
    }
}

fn make_true() -> bool {
    true
}

pub struct Config {
    pub general: General,
    #[cfg(feature = "debconf")]
    pub debconf: Option<::debconf::DebConfig>,
    pub defaults: Defaults,
    pub params: Vec<Param>,
    pub switches: Vec<Switch>,
}

#[derive(Debug)]
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct General {
    /// Name of the program
    pub name: Option<String>,

    /// Short description of the program
    pub summary: Option<String>,

    /// Long description of the program
    pub doc: Option<String>,

    /// Prefix for all env vars - enables
    /// all env vars by default if present
    pub env_prefix: Option<String>,

    /// The name of the parameter which, if
    /// specified causes parameter parsing to
    /// immediately load a config file, parse
    /// it, and override all configuration
    /// provided so far with that file.
    pub conf_file_param: Option<Ident>,

    /// The name of the parameter which, if
    /// specified causes parameter parsing to
    /// immediately load all files from the
    /// directory, parse them, and override all
    /// configuration provided so far with them.
    pub conf_dir_param: Option<Ident>,
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    #[serde(default = "make_true")]
    pub args: bool,
    #[serde(default)]
    pub env_vars: Option<bool>,
    #[serde(default = "make_true")]
    pub optional: bool,
}

impl Default for Defaults {
    fn default() -> Self {
        Defaults {
            args: true,
            env_vars: None,
            optional: true,
        }
    }
}

pub enum Optionality {
    Mandatory,
    Optional,
    DefaultValue(String),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SwitchKind {
    Normal { abbr: Option<char>, count: bool },
    Inverted,
}

pub struct Param {
    pub name: Ident,
    pub abbr: Option<char>,
    pub ty: String,
    pub optionality: Optionality,
    pub doc: Option<String>,
    pub argument: bool,
    pub env_var: bool,
    pub convert_into: String,
    pub merge_fn: Option<String>,
    #[cfg(feature = "debconf")]
    pub debconf_priority: Option<::debconf::Priority>,
    #[cfg(feature = "debconf")]
    pub debconf_default: Option<String>,
}

pub struct Switch {
    pub name: Ident,
    pub kind: SwitchKind,
    pub doc: Option<String>,
    pub env_var: bool,
    #[cfg(feature = "debconf")]
    pub debconf_priority: Option<::debconf::Priority>,
}

impl Switch {
    pub fn is_inverted(&self) -> bool {
        self.kind == SwitchKind::Inverted
    }

    pub fn is_count(&self) -> bool {
        if let SwitchKind::Normal { count: true, .. } = self.kind {
            true
        } else {
            false
        }
    }

}
