use std::fmt;

#[derive(Debug)]
pub enum FieldError {
    MandatoryWithDefault { optional_span: Span, default_span: Span, },
    InvertedWithAbbr { default_span: Span, abbr_span: Span, },
    InvertedWithCount { default_span: Span, count_span: Span, },
    InvalidAbbr { abbr_span: Span, },
    ReservedParameter,
}

#[derive(Debug)]
pub struct ValidationError {
    source: ValidationErrorSource,
}

impl ValidationError {
    // We sort by the span.start of the first root cause
    fn sort_key(&self) -> usize {
        use self::FieldError::*;
        use self::ValidationErrorSource::*;

        match &self.source {
            InvalidField { kind: MandatoryWithDefault { optional_span, default_span }, .. } => optional_span.start.min(default_span.start),
            InvalidField { kind: InvertedWithAbbr { abbr_span, default_span }, .. } => abbr_span.start.min(default_span.start),
            InvalidField { kind: InvertedWithCount { count_span, default_span }, .. } => count_span.start.min(default_span.start),
            InvalidField { kind: InvalidAbbr { abbr_span }, .. } => abbr_span.start,
            InvalidField { kind: ReservedParameter, span, .. } => span.start,
            Duplicates { duplicate_spans, .. } => duplicate_spans[0].start, // always non-empty
            InvalidIdentifier(error) => error.span().start,
            InvalidProgramName { span, .. } => span.start,
        }
    }
}

#[cfg_attr(not(feature = "spanned"), allow(unused))]
#[derive(Debug)]
enum ValidationErrorSource {
    InvalidField { name: String, span: Span, kind: FieldError },
    Duplicates { name: String, first_span: Span, duplicate_spans: Vec<Span> },
    InvalidIdentifier(ident::Error),
    InvalidProgramName { input: String, span: Span }
}

impl From<ident::Error> for ValidationError {
    fn from(value: ident::Error) -> Self {
        ValidationError { source: ValidationErrorSource::InvalidIdentifier(value) }
    }
}


impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::FieldError::*;
        use self::ValidationErrorSource::*;

        match &self.source {
            InvalidField { name, kind, .. } => {
                let msg = match kind {
                    MandatoryWithDefault { .. } => "parameter with default value must be optional",
                    InvertedWithAbbr { .. } => "inverted switch can't have a short option",
                    InvertedWithCount { .. } => "inverted switch can't be a count",
                    InvalidAbbr { .. } => "invalid short switch: must be [a-zA-Z]",
                    ReservedParameter => "this parameter is reserved and always implemented by configure_me",
                };
                write!(f, "invalid configuration for field {}: {}", name, msg)
            },
            // first span is stored separately so we have to add 1
            Duplicates { name, duplicate_spans, .. } => write!(f, "the option {} occurs {} times", name, duplicate_spans.len() + 1),
            InvalidIdentifier(error) => fmt::Display::fmt(error, f),
            InvalidProgramName { input, .. } => write!(f, "the string `{}` is not a valid program name handling strategy", input),
        }
    }
}

#[cfg(feature = "spanned")]
impl ValidationError {
    pub fn to_diagnostic<T: Copy>(&self, file_id: T) -> codespan_reporting::diagnostic::Diagnostic<T> {
        use self::FieldError::*;
        use codespan_reporting::diagnostic::Label;

        let diagnostic = codespan_reporting::diagnostic::Diagnostic::error();
        match &self.source {
            ValidationErrorSource::InvalidField { name, span, kind } => {
                match kind {
                    MandatoryWithDefault { optional_span, default_span } => {
                        diagnostic
                            .with_message("parameter attempts to be both optional and mandatory at the same time")
                            .with_labels(vec![
                                 Label::primary(file_id, *optional_span).with_message("setting `optional` to `false` makes the parameter mandatory here"),
                                 Label::primary(file_id, *default_span).with_message("the default value is provided here making the parameter optional"),
                                 Label::secondary(file_id, *span).with_message(format!("in the parameter `{}`", name)),
                            ])
                            .with_notes(vec![
                                "Help: either make the parameter optional or remove the default value".to_owned()
                            ])
                    },
                    InvertedWithAbbr { default_span, abbr_span } => {
                        diagnostic
                            .with_message("an inverted switch attempts to have a short option")
                            .with_labels(vec![
                                 Label::primary(file_id, *abbr_span).with_message("short option defined here"),
                                 Label::primary(file_id, *default_span).with_message("the default value is set to `true` here making the switch inverted"),
                                 Label::secondary(file_id, *span).with_message(format!("in the parameter `{}`", name)),
                            ])
                            .with_notes(vec![
                                "Help: remove the short option if you want to keep the parameter inverted".to_owned()
                            ])
                    },
                    InvertedWithCount { default_span, count_span } => {
                        diagnostic.with_message("inverted switch attempts to be a counter")
                            .with_labels(vec![
                                 Label::primary(file_id, *count_span).with_message("counter defined here"),
                                 Label::primary(file_id, *default_span).with_message("the default value is set to `true` here making the switch inverted"),
                                 Label::secondary(file_id, *span).with_message(format!("in the parameter `{}`", name)),
                            ])
                            .with_notes(vec![
                                "Help: either don't make the parameter counter or make the parameter non-inverted".to_owned()
                            ])
                    },
                    InvalidAbbr { abbr_span } => {
                        diagnostic
                            .with_message("invalid short option")
                            .with_labels(vec![
                                 Label::primary(file_id, *abbr_span).with_message("this option uses an invalid character"),
                                 Label::secondary(file_id, *span).with_message("in this field"),
                            ])
                            .with_notes(vec![
                                "Note: only English letters (both lower case and upper case) are allowed".to_owned()
                            ])
                    },
                    ReservedParameter => {
                        diagnostic
                            .with_message("use of reserved option")
                            .with_labels(vec![
                                 Label::primary(file_id, *span).with_message("this option is reserved because it's always implemented by `configure_me`"),
                            ])
                    },
                }
            },
            ValidationErrorSource::Duplicates { first_span, duplicate_spans, name } => {
                let mut labels = Vec::with_capacity(duplicate_spans.len() + 1);
                labels.push(Label::secondary(file_id, *first_span).with_message("the option was first defined here"));
                let mut iter = duplicate_spans.iter();
                let first_dup_span = *iter.next().expect("at least one duplicate");
                labels.push(Label::primary(file_id, first_dup_span).with_message("the option is repeated here"));
                labels.extend(iter.map(|span| Label::primary(file_id, *span).with_message("... and here")));
                diagnostic
                    .with_message(format!("the option `{}` appears more than once", name))
                    .with_labels(labels)
            },
            ValidationErrorSource::InvalidIdentifier(error) => error.to_diagnostic(file_id),
            ValidationErrorSource::InvalidProgramName { input, span } => {
                let labels = vec![Label::primary(file_id, *span).with_message("this is an invalid program name handling strategy")];
                diagnostic.with_message(format!("`{}` is not a valid program name handling strategy", input))
                    .with_labels(labels)
                    .with_notes(vec![
                        "Help: valid program name handling strategies are `unused`, `optional`, `required`.".to_owned()
                    ])
            },
        }
    }
}

mod ident {
    use std::convert::TryFrom;
    use std::fmt::{self, Write};
    use super::Span;
    use toml::Spanned;

    #[derive(Debug)]
    pub enum Error {
        InvalidChars {
            string: String,
            positions: Vec<usize>,
            span: Span,
        },
        Empty { span: Span, }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::InvalidChars { string, positions, .. } if positions.len() == 1 => write!(f, "\"{}\" is not a valid identifier, invalid char at position {}", string, positions[0]),
                Error::InvalidChars { string, positions, .. } => {
                    let mut iter = positions.iter();
                    write!(f, "\"{}\" is not a valid identifier, invalid chars at positions: {}", string, iter.next().expect("always at least one"))?;
                    for pos in iter {
                        write!(f, ", {}", pos)?;
                    }
                    Ok(())
                },
                Error::Empty { .. } => write!(f, "the identifier is empty"),
            }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct Ident(String);

    impl TryFrom<Spanned<String>> for Ident {
        type Error = Error;

        fn try_from(string: Spanned<String>) -> Result<Ident, Error> {
            let span = Span::from(&string);
            let string = string.into_inner();

            if string.is_empty() {
                return Err(Error::Empty { span, })
            }

            match Self::validate(&string) {
                Ok(()) => Ok(Ident(string)),
                Err(positions) => Err(Error::InvalidChars {
                    string,
                    positions,
                    span,
                }),
            }
        }
    }

    impl Ident {
        fn validate(string: &str) -> Result<(), Vec<usize>> {
            let invalid_chars_positions = string
                .chars()
                .enumerate()
                .filter(|&(i, c)| c != '_' && ! ((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9' && i > 0)))
                .map(|(i, _)| i)
                .collect::<Vec<_>>();

            if invalid_chars_positions.is_empty() {
                Ok(())
            } else {
                Err(invalid_chars_positions)
            }
        }

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

    impl Error {
        pub fn span(&self) -> Span {
            match self {
                Error::InvalidChars { span, .. } => *span,
                Error::Empty { span, .. } => *span,
            }
        }

        #[cfg(feature = "spanned")]
        pub fn to_diagnostic<T: Copy>(&self, file_id: T) -> codespan_reporting::diagnostic::Diagnostic<T> {
            use codespan_reporting::diagnostic::Label;

            let create_label = |start, end, was_emitted| {
                let label = Label::primary(file_id, start..end);
                match (end - start > 1, was_emitted) {
                    (false, false) => label.with_message("this char is invalid"),
                    (false, true) => label.with_message("... and this char"),
                    (true, false) => label.with_message("these chars are invalid"),
                    (true, true) => label.with_message("... and these chars"),
                }
            };

            match self {
                Error::InvalidChars { string, positions, span } => {
                    let offset = if string.len() == span.end - span.start {
                        // no quotes
                        0
                    } else {
                        // quotes were used so we have to add 1 to spans
                        1
                    };

                    // this may over-allocate but it's better to be fast than memory-saving
                    let mut labels = Vec::with_capacity(positions.len());
                    let mut positions = positions.iter();
                    let diagnostic = codespan_reporting::diagnostic::Diagnostic::error();
                    let diagnostic = if positions.len() > 1 {
                        diagnostic.with_message(format!("the identifier `{}` contains invalid characters", string))
                    } else {
                        diagnostic.with_message(format!("the identifier `{}` contains an invalid character", string))
                    };

                    // first one is special
                    let diagnostic = if string.starts_with(|c| c >= '0' && c <= '9') {
                        labels.push(Label::primary(file_id, (span.start + offset)..(span.start + 1 + offset)).with_message("the identifier starts with a digit"));
                        positions.next().expect("starting with zero is recorded");
                        diagnostic.with_notes(vec!["Help: identifiers mut not start with digits".to_owned()])
                    } else if string.starts_with('-') {
                        diagnostic.with_notes(vec!["Help: dashes are prepended automatically, you don't need to write them".to_owned()])
                    } else {
                        diagnostic
                    };

                    let contains_dashes_not_at_start = string
                        .chars()
                        .skip_while(|c| *c == '-')
                        .any(|c| c == '-');

                    let diagnostic = if contains_dashes_not_at_start {
                        diagnostic.with_notes(vec!["Help: consider replacing dashes with underscores.\n      They will be replaced with dashes in command line parameters\n      but stay underscores in config files.".to_owned()])
                    } else {
                        diagnostic
                    };

                    if let Some(first) = positions.next() {
                        let mut last_start = *first;
                        let mut last_end = *first + 1;
                        let mut was_emitted = false;
                        for position in positions {
                            if *position == last_end {
                                last_end += 1;
                            } else {
                                labels.push(create_label(span.start + last_start + offset, span.start + last_end + offset, was_emitted));
                                was_emitted = true;
                                last_start = *position;
                                last_end = *position + 1;
                            }
                        }
                        labels.push(create_label(span.start + last_start + offset, span.start + last_end + offset, was_emitted));
                    }

                    diagnostic
                        .with_labels(labels)
                },
                Error::Empty { span } => {
                    codespan_reporting::diagnostic::Diagnostic::error()
                        .with_message("encountered an empty identifier")
                        .with_labels(vec![
                                     Label::primary(file_id, *span)
                                         .with_message("this identifier is empty")
                        ])
                },
            }
        }
    }
}

use self::ident::Ident;

#[derive(Debug, Copy, Clone)]
pub struct Span {
    start: usize,
    end: usize,
}

impl From<Span> for core::ops::Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

pub mod raw {
    use toml::Spanned;
    use std::convert::TryFrom;
    use std::fmt;
    use super::{ValidationError, FieldError, ValidationErrorSource, Optionality, SwitchKind};
    use super::ident::Ident;
    use super::Span;

    #[derive(Debug)]
    struct Kv<V>(Spanned<String>, V);

    #[derive(Debug)]
    struct Map<V>(Vec<Kv<V>>);

    impl<V> Default for Map<V> {
        fn default() -> Self {
            Map(Default::default())
        }
    }

    impl<'de, V: serde::Deserialize<'de>> serde::Deserialize<'de> for Map<V> {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct Visitor<V>(std::marker::PhantomData<fn() -> V>);

            impl<'de, V: serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<V> {
                type Value = Map<V>;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("a map")
                }

                fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    let mut vec = Vec::with_capacity(map.size_hint().unwrap_or(0));

                    while let Some((key, value)) = map.next_entry()? {
                        vec.push(Kv(key, value));
                    }

                    Ok(Map(vec))
                }
            }

            deserializer.deserialize_map(Visitor(Default::default()))
        }
    }

    impl<V> IntoIterator for Map<V> {
        type Item = Kv<V>;
        type IntoIter = std::vec::IntoIter<Kv<V>>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<'a, T> From<&'a Spanned<T>> for Span {
        fn from(value: &'a Spanned<T>) -> Self {
            Span {
                start: value.start(),
                end: value.end(),
            }
        }
    }

    trait ResultExt {
        type Item;

        fn field_name(self, name: &Spanned<String>) -> Result<Self::Item, ValidationError>;
    }

    impl<T> ResultExt for Result<T, FieldError> {
        type Item = T;

        fn field_name(self, name: &Spanned<String>) -> Result<Self::Item, ValidationError> {
            let span = Span::from(name);
            self.map_err(|kind| ValidationError { source: ValidationErrorSource::InvalidField { name: name.get_ref().clone(), span, kind }})
        }
    }

    struct ArgValidator<T: Eq + std::hash::Hash> {
        // just for checking, order/determinism doesn't matter
        map: std::collections::HashMap<T, Option<Span>>,
        dup_cache: std::collections::HashMap<T, (Span, Vec<Span>)>,
    }

    impl<T: Eq + std::hash::Hash + Clone> ArgValidator<T> {
        fn with_reserved(arg: T) -> Self {
            let mut map = std::collections::HashMap::new();
            map.insert(arg, None);

            ArgValidator {
                map,
                dup_cache: Default::default(),
            }
        }

        fn check_insert(&mut self, arg: &Spanned<T>) -> Result<(), FieldError> {
            use std::collections::hash_map::Entry;

            match self.map.entry(arg.get_ref().clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(Some(Span::from(arg)));
                    Ok(())
                },
                Entry::Occupied(entry) => {
                    match entry.get() {
                        Some(span) => {
                            let dups = self.dup_cache.entry(arg.get_ref().clone()).or_insert_with(|| (*span, Vec::new()));
                            dups.1.push(arg.to_span());
                            Ok(())
                        },
                        None => Err(FieldError::ReservedParameter)
                    }
                }
            }
        }

        fn into_duplicates<S: 'static + FnMut(T) -> String>(self, mut stringify: S) -> impl Iterator<Item=ValidationError> {
            self.dup_cache.into_iter().map(move |(name, (first_span, duplicate_spans))| {
                ValidationError {
                    source: ValidationErrorSource::Duplicates {
                        name: stringify(name),
                        first_span,
                        duplicate_spans,
                    }
                }
            })
        }
    }

    impl ArgValidator<String> {
        fn check_insert_long(&mut self, arg: &Spanned<String>) -> Result<(), ValidationError> {
            self.check_insert(arg).field_name(arg)
        }

        fn check_insert_opt_long(&mut self, arg: &Option<Spanned<String>>) -> Result<(), ValidationError> {
            if let Some(arg) = arg {
                self.check_insert_long(arg)
            } else {
                Ok(())
            }
        }
    }

    pub trait IntoParts: Sized {
        type Value;

        fn into_parts(self) -> (Self::Value, Span);
        fn to_span(&self) -> Span;
        fn get(&self) -> Self::Value where Self::Value: Copy;

        fn to_parts(&self) -> (Self::Value, Span) where Self::Value: Copy {
            (self.get(), self.to_span())
        }
    }

    impl<T> IntoParts for Spanned<T> {
        type Value = T;

        fn into_parts(self) -> (Self::Value, Span) {
            let span = Span::from(&self);
            (self.into_inner(), span)
        }

        fn to_span(&self) -> Span {
            Span::from(self)
        }

        fn get(&self) -> Self::Value where Self::Value: Copy {
            *self.get_ref()
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Config {
        #[serde(rename = "param")]
        #[serde(default)]
        params: Map<Param>,
        #[serde(rename = "switch")]
        #[serde(default)]
        switches: Map<Switch>,
        #[serde(default)]
        general: General,
        #[serde(default)]
        defaults: super::Defaults,
        #[cfg(feature = "debconf")]
        debconf: Option<::debconf::DebConfig>,
    }

    impl Config {
        pub fn validate(self) -> Result<super::Config, Vec<ValidationError>> {
            use super::ProgramName;

            let default_optional = self.defaults.optional;
            let default_argument = self.defaults.args;
            let default_env_var = self.defaults.env_vars.unwrap_or(self.general.env_prefix.is_some());
            let mut errors = Vec::new();
            let mut long_args = ArgValidator::with_reserved("help".to_owned());
            let mut short_args = ArgValidator::with_reserved('h');

            long_args.check_insert_opt_long(&self.general.conf_file_param).unwrap_or_else(|error| errors.push(error));
            long_args.check_insert_opt_long(&self.general.conf_dir_param).unwrap_or_else(|error| errors.push(error));
            long_args.check_insert_opt_long(&self.general.skip_default_conf_files_switch).unwrap_or_else(|error| errors.push(error));

            let params = self.params
                .into_iter()
                .filter_map(|param| {
                    long_args.check_insert_long(&param.0).unwrap_or_else(|error| errors.push(error));
                    if let Some(abbr) = &param.1.abbr {
                        short_args.check_insert(abbr).field_name(&param.0).unwrap_or_else(|error| errors.push(error));
                    }
                    param.validate_param(default_optional, default_argument, default_env_var).map_err(|error| errors.extend(error)).ok()
                })
                .collect::<Vec<_>>();

            let switches = self.switches
                .into_iter()
                .filter_map(|switch| {
                    long_args.check_insert_long(&switch.0).unwrap_or_else(|error| errors.push(error));
                    if let Some(abbr) = &switch.1.abbr {
                        short_args.check_insert(abbr).field_name(&switch.0).unwrap_or_else(|error| errors.push(error));
                    }
                    switch.validate_switch(default_env_var).map_err(|error| errors.extend(error)).ok()
                })
                .collect::<Vec<_>>();

            errors.extend(long_args.into_duplicates(std::convert::identity));
            errors.extend(short_args.into_duplicates(|c| c.to_string()));

            let mut to_ident = |opt: Option<Spanned<String>>| {
                opt.and_then(|string| {
                    Ident::try_from(string).map_err(|error| errors.push(error.into())).ok()
                })
            };

            let conf_file_param = to_ident(self.general.conf_file_param);
            let conf_dir_param = to_ident(self.general.conf_dir_param);
            let skip_default_conf_files_switch = to_ident(self.general.skip_default_conf_files_switch);

            let program_name = self.general.program_name.map(|program_name| {
                let span = Span::from(&program_name);
                let program_name = program_name.into_inner();
                match &*program_name {
                    "unused" => ProgramName::Unused,
                    "optional" => ProgramName::Optional,
                    "required" => ProgramName::Required,
                    _ => {
                        let error = ValidationError {
                            source: ValidationErrorSource::InvalidProgramName { input: program_name, span },
                        };
                        errors.push(error);
                        ProgramName::Unused
                    }
                }
            }).unwrap_or_default();
            if !errors.is_empty() {
                errors.sort_by_key(ValidationError::sort_key);
                return Err(errors);
            }

            let general = super::General {
                name: self.general.name,
                summary: self.general.summary,
                doc: self.general.doc,
                env_prefix: self.general.env_prefix,
                conf_file_param,
                conf_dir_param,
                skip_default_conf_files_switch,
                program_name,
            };

            Ok(super::Config {
                general,
                defaults: self.defaults,
                params,
                switches,
                #[cfg(feature = "debconf")]
                debconf: self.debconf,
            })
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    pub struct General {
        name: Option<String>,
        summary: Option<String>,
        doc: Option<String>,
        env_prefix: Option<String>,
        conf_file_param: Option<Spanned<String>>,
        conf_dir_param: Option<Spanned<String>>,
        skip_default_conf_files_switch: Option<Spanned<String>>,
        program_name: Option<Spanned<String>>,
    }


    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Param {
        abbr: Option<Spanned<char>>,
        #[serde(rename = "type")]
        ty: String,
        optional: Option<Spanned<bool>>,
        default: Option<Spanned<String>>,
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
        fn validate_optionality(optional: Option<Spanned<bool>>, default_optional: bool, default: Option<Spanned<String>>) -> Result<Optionality, FieldError> {
            match (optional, default_optional, default) {
                (Some(opt), _, None) if !opt.get() => Ok(Optionality::Mandatory),
                (Some(opt), _, Some(default)) if !opt.get() => Err(FieldError::MandatoryWithDefault { optional_span: opt.to_span(), default_span: default.to_span(), }),
                (Some(_), _, None) => Ok(Optionality::Optional),
                (_, _, Some(default)) => Ok(Optionality::DefaultValue(default.into_inner())),
                (None, true, None) => Ok(Optionality::Optional),
                (None, false, None) => Ok(Optionality::Mandatory),
            }
        }
    }

    impl Kv<Param> {
        fn validate_param(self, default_optional: bool, default_argument: bool, default_env_var: bool) -> Result<super::Param, impl Iterator<Item=ValidationError>> {
            let optionality = Param::validate_optionality(self.1.optional, default_optional, self.1.default)
                .field_name(&self.0);
            let name = Ident::try_from(self.0).map_err(Into::into);

            let (name, optionality) = match (name, optionality) {
                (Ok(name), Ok(optionality)) => (name, optionality),
                (err1, err2) => return Err(err1.err().into_iter().chain(err2.err())),
            };

            let ty = self.1.ty;
            let argument = self.1.argument.unwrap_or(default_argument);
            let env_var = self.1.env_var.unwrap_or(default_env_var);
            let needs_conversion = self.1.convert_into.is_some();
            let convert_into = self.1.convert_into.unwrap_or_else(|| ty.clone());

            Ok(super::Param {
                name,
                ty,
                optionality,
                abbr: self.1.abbr.map(Spanned::into_inner),
                doc: self.1.doc,
                argument,
                env_var,
                convert_into,
                needs_conversion,
                merge_fn: self.1.merge_fn,
                #[cfg(feature = "debconf")]
                debconf_priority: self.1.debconf_priority,
                #[cfg(feature = "debconf")]
                debconf_default: self.1.debconf_default,
            })
        }
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Switch {
        abbr: Option<Spanned<char>>,
        default: Option<Spanned<bool>>,
        doc: Option<String>,
        env_var: Option<bool>,
        count: Option<Spanned<bool>>,
        #[cfg(feature = "debconf")]
        debconf_priority: Option<::debconf::Priority>,
    }

    impl Switch {
        fn validate_abbr(spanned_abbr: Spanned<char>) -> Result<Spanned<char>, FieldError> {
            let (abbr, abbr_span) = spanned_abbr.to_parts();
            if (abbr >= 'a' && abbr <= 'z') || (abbr >= 'A' && abbr <= 'Z') {
                Ok(spanned_abbr)
            } else {
                Err(FieldError::InvalidAbbr { abbr_span, })
            }
        }

        fn validate_kind(abbr: Option<Spanned<char>>, default: Option<Spanned<bool>>, count: Option<Spanned<bool>>) -> Result<SwitchKind, FieldError> {
            match (abbr, default, count) {
                (Some(abbr), Some(default), _) if default.get() => Err(FieldError::InvertedWithAbbr { abbr_span: abbr.to_span(), default_span: default.to_span() }),
                (_, Some(default), Some(count)) if default.get() && count.get() => Err(FieldError::InvertedWithCount { default_span: default.to_span(), count_span: count.to_span() }),
                (None, Some(default), _) if default.get() => Ok(SwitchKind::Inverted),
                (abbr, _, count) => Ok(SwitchKind::Normal { abbr: abbr.map(Spanned::into_inner), count: count.map(Spanned::into_inner).unwrap_or(false) }),
            }
        }
    }

    impl Kv<Switch> {
        fn validate_switch(self, default_env_var: bool) -> Result<super::Switch, impl Iterator<Item=ValidationError>> {
            let abbr = self.1.abbr;
            let default = self.1.default;
            let count = self.1.count;
            let name = &self.0;

            let kind = abbr
                .map(Switch::validate_abbr)
                .transpose()
                .field_name(name)
                .and_then(|abbr| {
                    Switch::validate_kind(abbr, default, count)
                        .field_name(name)
                });

            let name = Ident::try_from(self.0).map_err(Into::into);

            let (name, kind) = match (name, kind) {
                (Ok(name), Ok(kind)) => (name, kind),
                (err1, err2) => return Err(err1.err().into_iter().chain(err2.err())),
            };

            Ok(super::Switch {
                name,
                kind,
                doc: self.1.doc,
                env_var: self.1.env_var.unwrap_or(default_env_var),
                #[cfg(feature = "debconf")]
                debconf_priority: self.1.debconf_priority,
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

#[derive(Debug, Default)]
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

    /// The name of the switch which, if
    /// specified, avoids reading default
    /// configuration files.
    pub skip_default_conf_files_switch: Option<Ident>,

    /// How to handle the zeroth argument - program name.
    ///
    /// If the program name is unused it will not be added to the `Metadata` struct.
    /// If the program name is optional the field `program_name: Option<PathBuf>` is
    /// added to `Metadata` struct.
    /// If the program name is required `PathBuf` is added to `Metadata` and a nice error message
    /// will be reported if it is missing.
    pub program_name: ProgramName,
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
    pub needs_conversion: bool,
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

#[derive(Debug)]
pub enum ProgramName {
    Unused,
    Optional,
    Required,
}

impl Default for ProgramName {
    fn default() -> Self {
        ProgramName::Unused
    }
}
