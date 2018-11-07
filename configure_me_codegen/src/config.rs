#[derive(Debug)]
pub enum ValidationErrorKind {
    MandatoryWithDefault,
    InvertedWithAbbr,
    InvalidAbbr,
}

#[derive(Debug)]
pub struct ValidationError {
    name: String,
    kind: ValidationErrorKind,
}

pub mod raw {
    use super::{ValidationError, ValidationErrorKind};

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
    }

    impl Config {
        pub fn validate(self) -> Result<super::Config, ValidationError> {
            let default_optional = self.defaults.optional;
            let default_argument = self.defaults.args;
            let default_env_var = self.defaults.env_vars;
            let params = self.params
                .into_iter()
                .map(|param| param.validate(default_optional, default_argument, default_env_var))
                .collect::<Result<Vec<_>, _>>()?;

            let switches = self.switches
                .into_iter()
                .map(Switch::validate)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(super::Config {
                general: self.general,
                defaults: self.defaults,
                params,
                switches,
            })
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Param {
        name: String,
        abbr: Option<String>,
        #[serde(rename = "type")]
        ty: String,
        optional: Option<bool>,
        default: Option<String>,
        doc: Option<String>,
        argument: Option<bool>,
        env_var: Option<bool>,
    }

    impl Param {
        fn validate(self, default_optional: bool, default_argument: bool, default_env_var: bool) -> Result<super::Param, ValidationError> {
            use super::Optionality;

            let optionality = match (self.optional, default_optional, self.default) {
                (Some(false), _, None) => Optionality::Mandatory,
                (Some(false), _, Some(_)) => return Err(ValidationError { name: self.name, kind: ValidationErrorKind::MandatoryWithDefault, }),
                (Some(true), _, None) => Optionality::Optional,
                (_, _, Some(default)) => Optionality::DefaultValue(default),
                (None, true, None) => Optionality::Optional,
                (None, false, None) => Optionality::Mandatory,
            };

            let abbr = if let Some(mut abbr) = self.abbr {
                let abbr_chr = abbr.pop();

                if abbr.len() > 0 {
                    return Err(ValidationError { name: self.name, kind: ValidationErrorKind::InvalidAbbr, });
                }
                Some(if let Some(abbr) = abbr_chr {
                    abbr
                } else {
                    return Err(ValidationError { name: self.name, kind: ValidationErrorKind::InvalidAbbr, });
                })
            } else {
                None
            };

            let argument = self.argument.unwrap_or(default_argument);
            let env_var = self.env_var.unwrap_or(default_env_var);

            Ok(super::Param {
                name: self.name,
                ty: self.ty,
                optionality,
                abbr,
                doc: self.doc,
                argument,
                env_var,
            })
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Switch {
        name: String,
        abbr: Option<String>,
        default: Option<bool>,
        doc: Option<String>,
    }

    impl Switch {
        fn validate(self) -> Result<super::Switch, ValidationError> {
            use super::SwitchKind;

            let kind = match (self.abbr, self.default) {
                (Some(_), Some(true)) => return Err(ValidationError { name: self.name, kind: ValidationErrorKind::InvertedWithAbbr, }),
                (Some(mut abbr), _) => match abbr.pop() {
                    Some(chr) if abbr.len() == 0 && ((chr >= 'a' && chr <= 'z') || (chr >= 'A' && chr <= 'Z')) => SwitchKind::WithAbbr(chr),
                    _ => return Err(ValidationError { name: self.name, kind: ValidationErrorKind::InvalidAbbr, }),
                },
                (None, Some(true)) => SwitchKind::Inverted,
                (None, _) => SwitchKind::Normal,
            };

            Ok(super::Switch {
                name: self.name,
                kind,
                doc: self.doc,
            })
        }
    }
}

fn make_true() -> bool {
    true
}

pub struct Config {
    pub general: General,
    pub defaults: Defaults,
    pub params: Vec<Param>,
    pub switches: Vec<Switch>,
}

#[derive(Debug)]
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct General {
    pub name: Option<String>,
    pub summary: Option<String>,
    pub doc: Option<String>,
    pub env_prefix: Option<String>,
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    #[serde(default = "make_true")]
    pub args: bool,
    #[serde(default = "make_true")]
    pub env_vars: bool,
    #[serde(default = "make_true")]
    pub optional: bool,
}

impl Default for Defaults {
    fn default() -> Self {
        Defaults {
            args: true,
            env_vars: true,
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
    Normal,
    WithAbbr(char),
    Inverted,
}

pub struct Param {
    pub name: String,
    pub abbr: Option<char>,
    pub ty: String,
    pub optionality: Optionality,
    pub doc: Option<String>,
    pub argument: bool,
    pub env_var: bool,
}

pub struct Switch {
    pub name: String,
    pub kind: SwitchKind,
    pub doc: Option<String>,
}

impl Switch {
    pub fn is_inverted(&self) -> bool {
        self.kind == SwitchKind::Inverted
    }
}
