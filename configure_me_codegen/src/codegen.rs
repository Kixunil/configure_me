use std::fmt::{self, Write};
use ::config::{Config, Optionality};
use ::unicode_segmentation::UnicodeSegmentation;

mod visitor {
    use std::fmt;

    pub trait VisitWrite<T> {
        fn visit_write<W: fmt::Write>(&self, output: W) -> fmt::Result;
    }

    impl<'a, T, U> VisitWrite<T> for &'a U where U: VisitWrite<T> {
        fn visit_write<W: fmt::Write>(&self, output: W) -> fmt::Result {
            (*self).visit_write(output)
        }
    }

    pub fn iter<T, I, W: fmt::Write>(iter: I, mut output: W) -> fmt::Result where I: IntoIterator, I::Item: VisitWrite<T> {
        for item in iter {
            item.visit_write(&mut output)?;
        }
        Ok(())
    }

    pub enum RawConfigDecl {}
    pub enum ArgParseErrorDecl {}
    pub enum EnvParseErrorDecl {}
    pub enum ConfigFinal {}
    pub enum Validate {}
    pub enum ConstructConfig {}
    pub enum MergeIn {}
    pub enum MergeArgs {}
    pub enum MergeShort {}
}

use self::visitor::VisitWrite;

macro_rules! empty {
    ($type:ty, $visit:ident) => {
        impl VisitWrite<visitor::$visit> for $type {
            fn visit_write<W: fmt::Write>(&self, _output: W) -> fmt::Result {
                Ok(())
            }
        }
    }
}

empty!(::config::General, RawConfigDecl);

impl VisitWrite<visitor::RawConfigDecl> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        writeln!(output, "        {}: Option<{}>,", self.name.as_snake_case(), self.ty)
    }
}

impl VisitWrite<visitor::RawConfigDecl> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if self.is_count() {
            writeln!(output, "        {}: Option<u32>,", self.name.as_snake_case())
        } else {
            writeln!(output, "        {}: Option<bool>,", self.name.as_snake_case())
        }
    }
}

empty!(::config::General, ArgParseErrorDecl);

impl VisitWrite<visitor::ArgParseErrorDecl> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if self.argument {
            writeln!(output, "    Field{}(<{} as ::configure_me::parse_arg::ParseArg>::Error),", self.name.as_pascal_case(), self.ty)
        } else {
            Ok(())
        }
    }
}

empty!(::config::Switch, ArgParseErrorDecl);

empty!(::config::General, EnvParseErrorDecl);

impl VisitWrite<visitor::EnvParseErrorDecl> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if self.env_var {
            writeln!(output, "    Field{}(<{} as ::configure_me::parse_arg::ParseArg>::Error),", self.name.as_pascal_case(), self.ty)
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::EnvParseErrorDecl> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if self.env_var {
            if self.is_count() {
                writeln!(output, "    Field{}(<u32 as ::configure_me::parse_arg::ParseArg>::Error),", self.name.as_pascal_case())
            } else {
                writeln!(output, "    Field{}(::std::ffi::OsString),", self.name.as_pascal_case())
            }
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::ConfigFinal> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        match self.optionality {
            Optionality::Optional => writeln!(output, "    pub {}: Option<{}>,", self.name.as_snake_case(), self.convert_into),
            _ => writeln!(output, "    pub {}: {},", self.name.as_snake_case(), self.convert_into),
        }
    }
}

impl VisitWrite<visitor::ConfigFinal> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if self.is_count() {
            writeln!(output, "    pub {}: u32,", self.name.as_snake_case())
        } else {
            writeln!(output, "    pub {}: bool,", self.name.as_snake_case())
        }
    }
}

impl VisitWrite<visitor::Validate> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        match self.optionality {
            Optionality::Optional => writeln!(output, "            let {} = self.{};", self.name.as_snake_case(), self.name.as_snake_case()),
            Optionality::Mandatory => writeln!(output, "            let {} = self.{}.ok_or(ValidationError::MissingField(\"{}\"))?;", self.name.as_snake_case(), self.name.as_snake_case(), self.name.as_snake_case()),
            Optionality::DefaultValue(ref val) => writeln!(output, "            let {} = self.{}.unwrap_or_else(|| {{ {} }});", self.name.as_snake_case(), self.name.as_snake_case(), val),
        }
    }
}

empty!(::config::Switch, Validate);

impl VisitWrite<visitor::ConstructConfig> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if let Optionality::Optional = self.optionality {
            writeln!(output, "                {}: {}.map(Into::into),", self.name.as_snake_case(), self.name.as_snake_case())
        } else {
            writeln!(output, "                {}: {}.into(),", self.name.as_snake_case(), self.name.as_snake_case())
        }
    }
}

impl VisitWrite<visitor::ConstructConfig> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        let default_value = if self.is_inverted() {
            "true"
        } else if self.is_count() {
            "0"
        } else {
            "false"
        };
        writeln!(output, "                {}: self.{}.unwrap_or({}),", self.name.as_snake_case(), self.name.as_snake_case(), default_value)
    }
}

impl VisitWrite<visitor::MergeIn> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if let Some(merge_fn) = &self.merge_fn {
            writeln!(output, "            if let Some({}) = other.{} {{", self.name.as_snake_case(), self.name.as_snake_case())?;
            writeln!(output, "                if let Some({}_old) = &mut self.{} {{", self.name.as_snake_case(), self.name.as_snake_case())?;
            writeln!(output, "                    {}({}_old, {});", merge_fn, self.name.as_snake_case(), self.name.as_snake_case())?;
            writeln!(output, "                }} else {{")?;
            writeln!(output, "                    self.{} = Some({});", self.name.as_snake_case(), self.name.as_snake_case())?;
            writeln!(output, "                }}")?;
            writeln!(output, "            }}")
        } else {
            writeln!(output, "            if other.{}.is_some() {{", self.name.as_snake_case())?;
            writeln!(output, "                self.{} = other.{};", self.name.as_snake_case(), self.name.as_snake_case())?;
            writeln!(output, "            }}")
        }
    }
}

impl VisitWrite<visitor::MergeIn> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        writeln!(output, "            if other.{}.is_some() {{", self.name.as_snake_case())?;
        writeln!(output, "                self.{} = other.{};", self.name.as_snake_case(), self.name.as_snake_case())?;
        writeln!(output, "            }}")
    }
}

impl VisitWrite<visitor::MergeArgs> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if self.argument {
            writeln!(output, "                }} else if let Some(value) = ::configure_me::parse_arg::match_arg(\"--{}\", &arg, &mut iter) {{", self.name.as_hypenated())?;
            writeln!(output, "                    let {} = value.map_err(|err| err.map_or(ArgParseError::MissingArgument(\"--{}\"), ArgParseError::Field{}))?;", self.name.as_snake_case(), self.name.as_hypenated(), self.name.as_pascal_case())?;
            writeln!(output)?;
            if let Some(merge_fn) = &self.merge_fn {
                writeln!(output, "                    if let Some({}_old) = &mut self.{} {{", self.name.as_snake_case(), self.name.as_snake_case())?;
                writeln!(output, "                        {}({}_old, {});", merge_fn, self.name.as_snake_case(), self.name.as_snake_case())?;
                writeln!(output, "                    }} else {{")?;
                writeln!(output, "                        self.{} = Some({});", self.name.as_snake_case(), self.name.as_snake_case())?;
                writeln!(output, "                    }}")
            } else {
                writeln!(output, "                    self.{} = Some({});", self.name.as_snake_case(), self.name.as_snake_case())
            }
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::MergeArgs> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if self.is_inverted() {
            writeln!(output, "                }} else if arg == *\"--no-{}\" {{", self.name.as_hypenated())?;
            writeln!(output, "                    self.{} = Some(false);", self.name.as_snake_case())
        } else {
            writeln!(output, "                }} else if arg == *\"--{}\" {{", self.name.as_hypenated())?;

            if self.is_count() {
                writeln!(output, "                    *(self.{}.get_or_insert(0)) += 1;", self.name.as_snake_case())
            } else {
                writeln!(output, "                    self.{} = Some(true);", self.name.as_snake_case())
            }
        }
    }
}

impl VisitWrite<visitor::MergeArgs> for ::config::General {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        if let Some(conf_file) = &self.conf_file_param {
            writeln!(output, "                }} else if let Some(value) = ::configure_me::parse_arg::match_arg(\"--{}\", &arg, &mut iter) {{", conf_file.as_hypenated())?;
            writeln!(output, "                    let file_path: std::path::PathBuf = value.map_err(|err| err.map_or(ArgParseError::MissingArgument(\"--{}\"), |never| match never {{}}))?;", conf_file.as_hypenated())?;
            writeln!(output, "                    let mut config = Config::load(file_path)?;")?;
            writeln!(output, "                    self.merge_in(config);")?;
        }

        if let Some(conf_dir) = &self.conf_dir_param {
            writeln!(output, "                }} else if let Some(value) = ::configure_me::parse_arg::match_arg(\"--{}\", &arg, &mut iter) {{", conf_dir.as_hypenated())?;
            writeln!(output, "                    let dir_path: std::path::PathBuf = value.map_err(|err| err.map_or(ArgParseError::MissingArgument(\"--{}\"), |never| match never {{}}))?;", conf_dir.as_hypenated())?;
            writeln!(output)?;
            writeln!(output, "                    let dir = match std::fs::read_dir(&dir_path) {{")?;
            writeln!(output, "                        Ok(dir) => dir,")?;
            writeln!(output, "                        Err(err) => return Err(ArgParseError::OpenConfDir(err, dir_path).into()),")?;
            writeln!(output, "                    }};")?;
            writeln!(output)?;
            writeln!(output, "                    for file in dir {{")?;
            writeln!(output, "                        let file = match file {{")?;
            writeln!(output, "                            Ok(file) => file,")?;
            writeln!(output, "                            Err(err) => return Err(ArgParseError::ReadConfDir(err, dir_path).into()),")?;
            writeln!(output, "                        }};")?;
            writeln!(output)?;
            writeln!(output, "                        let mut config = Config::load(file.path())?;")?;
            writeln!(output, "                        self.merge_in(config);")?;
            writeln!(output, "                    }}")?;
        }
        Ok(())
    }
}

impl VisitWrite<visitor::MergeShort> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        // TODO remove invalid case (false, Some(_))
        if let (true, Some(short) )= (self.argument, self.abbr) {
            writeln!(output, "                        }} else if short == '{}' {{", short)?;
            writeln!(output, "                            self.{} = Some(shorts.parse_remaining(&mut iter).map_err(|err| err.map_or(ArgParseError::MissingArgument(\"-{}\"), ArgParseError::Field{}))?);", &self.name.as_snake_case(), short, self.name.as_pascal_case())?;
            writeln!(output, "                            break;")
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::MergeShort> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, mut output: W) -> fmt::Result {
        use ::config::SwitchKind;

        if let SwitchKind::Normal { abbr: Some(abbr), count } = &self.kind {
            writeln!(output, "                        }} else if short == '{}' {{", abbr)?;

            if *count {
                writeln!(output, "                            *(self.{}.get_or_insert(0)) += 1;", self.name.as_snake_case())
            } else {
                writeln!(output, "                            self.{} = Some(true);", self.name.as_snake_case())
            }
        } else {
            Ok(())
        }
    }
}

empty!(::config::General, MergeShort);

pub(crate) fn param_long_raw(param: &str) -> String {
    let mut res = String::with_capacity(param.len() + 2);
    res.push_str("--");
                                            // Writing to String never fails
    underscore_to_hypen(&mut res, &param).unwrap();

    res
}

pub(crate) fn param_long(param: &::config::Param) -> String {
    param_long_raw(&param.name.as_snake_case())
}

pub(crate) fn switch_long(switch: &::config::Switch) -> String {
    if switch.is_inverted() {
        let mut res = String::with_capacity(switch.name.as_snake_case().len() + 5);
                                            // Writing to String never fails
        write!(res, "--no-{}", switch.name.as_snake_case()).unwrap();
        res
    } else {
        let mut res = String::with_capacity(switch.name.as_snake_case().len() + 2);
                                            // Writing to String never fails
        write!(res, "--{}", switch.name.as_snake_case()).unwrap();
        res
    }
}

pub(crate) fn param_short(param: &::config::Param) -> Option<String> {
    let abbr = param.abbr?;
    let mut res = String::with_capacity(2);
    res.push('-');
    res.push(abbr);
    Some(res)
}

pub(crate) fn switch_short(switch: &::config::Switch) -> Option<String> {
    if let ::config::SwitchKind::Normal { abbr: Some(abbr), .. } = switch.kind {
        let mut res = String::with_capacity(2);
        res.push('-');
        res.push(abbr);
        Some(res)
    } else {
        None
    }
}

fn upper_case<W: Write>(mut output: W, string: &str) -> fmt::Result {
    for ch in string.chars().flat_map(char::to_uppercase) {
        write!(output, "{}", ch)?;
    }
    Ok(())
}

fn write_params_and_switches<T, W: Write>(config: &Config, mut output: W) -> fmt::Result where ::config::Param: VisitWrite<T>, ::config::Switch: VisitWrite<T> {
    visitor::iter::<T, _, _>(&config.params, &mut output)?;
    visitor::iter::<T, _, _>(&config.switches, &mut output)?;
    Ok(())
}

fn write_config<T, W: Write>(config: &Config, mut output: W) -> fmt::Result where ::config::Param: VisitWrite<T>, ::config::Switch: VisitWrite<T>, ::config::General: VisitWrite<T> {
    VisitWrite::<T>::visit_write(&config.general, &mut output)?;
    write_params_and_switches::<T, _>(config, &mut output)
}

fn gen_raw_config<W: Write>(config: &Config, output: W) -> fmt::Result {
    write_params_and_switches::<visitor::RawConfigDecl, _>(config, output)
}

fn gen_arg_parse_error<W: Write>(config: &Config, mut output: W) -> fmt::Result {
    write_params_and_switches::<visitor::ArgParseErrorDecl, _>(config, &mut output)?;
    if config.general.conf_dir_param.is_some() {
        writeln!(output, "    OpenConfDir(std::io::Error, std::path::PathBuf),")?;
        writeln!(output, "    ReadConfDir(std::io::Error, std::path::PathBuf),")?;
    }
    Ok(())
}

fn gen_env_parse_error<W: Write>(config: &Config, output: W) -> fmt::Result {
    write_params_and_switches::<visitor::EnvParseErrorDecl, _>(config, output)
}

fn gen_display_arg_parse_error<W: Write>(config: &Config, mut output: W) -> fmt::Result {
    use ::config::SwitchKind;

    let sum_arg_len = config
        .params
        .iter()
        .filter(|param| param.argument)
        .map(|param| param.name.as_snake_case().len() * 2 + 6)
        .sum::<usize>()
        + config
        .switches
        .iter()
        .map(|switch| switch.name.as_snake_case().len() + if switch.is_inverted() { 8 } else { 5 })
        .sum::<usize>()
        + config
        .general
        .conf_file_param
        .as_ref()
        .map(|param| param.as_snake_case().len() + 6 + 9)
        .unwrap_or(0)
        + config
        .general
        .conf_dir_param
        .as_ref()
        .map(|param| param.as_snake_case().len() + 6 + 8)
        .unwrap_or(0);

    write!(output, "        ArgParseError::HelpRequested(program_name) => write!(f, \"Usage: {{}}")?;
    // Standard width of the terminal - "Usage: ".len()
    if sum_arg_len < (80 - 7) {
        if let Some(conf_file_param) = &config.general.conf_file_param {
            write!(output, " [--{} CONF_FILE]", conf_file_param.as_hypenated())?;
        }
        if let Some(conf_dir_param) = &config.general.conf_dir_param {
            write!(output, " [--{} CONF_DIR]", conf_dir_param.as_hypenated())?;
        }
        for param in config.params.iter().filter(|param| param.argument) {
            if let Some(abbr) = &param.abbr {
                write!(output, " [-{} {}|--", abbr, param.name.as_upper_case())?;
            } else {
                write!(output, " [--")?;
            }
            write!(output, "{} {}]", param.name.as_hypenated(), param.name.as_upper_case())?;
        }
        for switch in config.switches.iter() {
            if let SwitchKind::Normal { abbr: Some(abbr), .. } = &switch.kind {
                write!(output, " [-{}|--", abbr)?;
            } else {
                write!(output, " [--")?;
            }
            if switch.is_inverted() {
                write!(output, "no-")?;
            }
            write!(output, "{}", switch.name.as_hypenated())?;
            if switch.is_count() {
                write!(output, " ...")?;
            }
            write!(output, "]")?;
        }
    } else {
        write!(output, " [ARGUMENTS...]")?;
    }
    let conf_files = config
        .general.conf_file_param
        .as_ref()
        .into_iter()
        .chain(config.general.conf_dir_param.as_ref())
        .map(|arg| arg.as_snake_case().len());

    let max_param_len = config
        .params
        .iter()
        .filter(|param| param.argument)
        .filter(|param| sum_arg_len > (80 - 7) || param.doc.is_some())
        .map(|param| param.name.as_snake_case().len() + if param.abbr.is_some() { 4 } else { 0 })
        .chain(conf_files)
        .max()
        .unwrap_or(0);
    let max_switch_len = config
        .switches
        .iter()
        .filter(|switch| sum_arg_len > (80 - 7) || switch.doc.is_some())
        .map(|switch| switch.name.as_snake_case().len() + match switch.kind {
            SwitchKind::Normal { abbr: Some(_), .. } => 4,
            SwitchKind::Inverted => 3,
            _ => 0,
        })
        .max()
        .unwrap_or(0);
    let max_arg_len = ::std::cmp::max(max_param_len, max_switch_len);
    let doc_start = 8 + 2 + max_arg_len + 4;
    if max_arg_len > 0 {
        write!(output, "\\n\\nArguments:")?;
        let conf_file = config
            .general.conf_file_param
            .as_ref()
            .map(|arg| (arg, Some("Load configuration from this file."), SwitchKind::Normal { abbr: None, count: false }))
            .into_iter();
        let conf_dir = config
            .general.conf_dir_param
            .as_ref()
            .map(|arg| (arg, Some("Load configuration from files in this directory."), SwitchKind::Normal { abbr: None, count: false }))
            .into_iter();

        let params = config
            .params
            .iter()
            .filter(|param| param.argument)
            .map(|param| (&param.name, param.doc.as_ref().map(AsRef::as_ref), SwitchKind::Normal { abbr: param.abbr, count: false }));
        let switches = config
            .switches
            .iter()
            .map(|switch| (&switch.name, switch.doc.as_ref().map(AsRef::as_ref), switch.kind));

        for (name, doc, switch_kind) in conf_file.chain(conf_dir).chain(params).chain(switches) {
            if let Some(doc) = doc {
                if doc.len() > 0 || sum_arg_len > (80 - 7) {
                    let name_len = match switch_kind {
                        SwitchKind::Normal { abbr: Some(abbr), .. } => {
                            write!(output, "\\n        -{}, --{}", abbr, name.as_hypenated())?;
                            name.as_snake_case().len() + 4
                        },
                        SwitchKind::Normal { abbr: None, .. } => {
                            write!(output, "\\n        --{}", name.as_hypenated())?;
                            name.as_snake_case().len()
                        },
                        SwitchKind::Inverted => {
                            write!(output, "\\n        --no-{}", name.as_hypenated())?;
                            name.as_snake_case().len() + 3
                        },
                    };

                    for _ in 0..(max_arg_len + 4 - name_len) {
                        output.write_char(' ')?;
                    }
                    let mut pos = doc_start;
                    for word in doc.split_word_bounds() {
                        let word_len = word.graphemes(true).count();
                        if word_len + pos > 80 {
                            write!(output, "\\n          ")?;
                            for _ in 0..(max_arg_len + 4) {
                                write!(output, " ")?;
                            }
                            pos = doc_start;
                        }

                        if !(word.trim().len() == 0 && pos ==  doc_start) {
                            write!(output, "{}", word)?;
                            pos += word_len;
                        }
                    }
                }
            } else if sum_arg_len > (80 - 7) {
                    match switch_kind {
                        SwitchKind::Normal { abbr: Some(abbr), .. } => write!(output, "\\n        -{}, --", abbr)?,
                        SwitchKind::Normal { abbr: None, .. } => write!(output, "\\n        --")?,
                        SwitchKind::Inverted => write!(output, "no-")?,
                    }

                    write!(output, "{}\\n", name.as_hypenated())?;
            }
        }
    }
    writeln!(output, "\", program_name),")?;
    for param in &config.params {
        if !param.argument {
            continue;
        }

        writeln!(output, "        ArgParseError::Field{}(err) => {{", param.name.as_pascal_case())?;
        writeln!(output, "            write!(f, \"Failed to parse argument '--{}': {{}}.\\n\\nHint: the value must be \", err)?;", param.name.as_hypenated())?;
        writeln!(output, "            <{} as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;", param.ty)?;
        writeln!(output, "            write!(f, \".\")")?;
        writeln!(output, "        }},")?;
    }
    if config.general.conf_dir_param.is_some() {
        writeln!(output, "        ArgParseError::OpenConfDir(err, dir) => write!(f, \"Failed to open configuration directory {{}}: {{}}\", dir.display(), err),")?;
        writeln!(output, "        ArgParseError::ReadConfDir(err, dir) => write!(f, \"Failed to read configuration directory {{}}: {{}}\", dir.display(), err),")?;
    }
    Ok(())
}

fn gen_display_env_parse_error<W: Write>(config: &Config, mut output: W) -> fmt::Result {
    for param in &config.params {
        if !param.env_var {
            continue;
        }

        writeln!(output, "        EnvParseError::Field{}(ref err) => {{", param.name.as_pascal_case())?;
        write!(output, "            write!(f, \"Failed to parse environment variable '")?;
        config.general.env_prefix.as_ref().map(|prefix| { upper_case(&mut output, &prefix)?; write!(output, "_") }).unwrap_or(Ok(()))?;
        writeln!(output, "{}': {{}}.\\n\\nHint: the value must be \", err)?;", param.name.as_upper_case())?;
        writeln!(output, "            <{} as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;", param.ty)?;
        writeln!(output, "            write!(f, \".\")")?;
        writeln!(output, "        }},")?;
    }
    for switch in &config.switches {
        if !switch.env_var {
            continue;
        }

        writeln!(output, "        EnvParseError::Field{}(ref err) => {{", switch.name.as_pascal_case())?;
        if switch.is_count() {
            write!(output, "            write!(f, \"Invalid value for '")?;
            config.general.env_prefix.as_ref().map(|prefix| { upper_case(&mut output, &prefix)?; write!(output, "_") }).unwrap_or(Ok(()))?;
            writeln!(output, "{}': {{}}.\\n\\nHint: the value must be \", err)?;", switch.name.as_upper_case())?;
            writeln!(output, "            <u32 as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;")?;
            writeln!(output, "            write!(f, \".\")")?;
        } else {
            write!(output, "            write!(f, \"Invalid value '{{:?}}' for '")?;
            config.general.env_prefix.as_ref().map(|prefix| { upper_case(&mut output, &prefix)?; write!(output, "_") }).unwrap_or(Ok(()))?;
            writeln!(output, "{}'.\\n\\nHint: the allowed values are 0, false, 1, true.\", err)", switch.name.as_upper_case())?;
        }
        writeln!(output, "        }},")?;
    }
    Ok(())
}

fn gen_validation_fn<W: Write>(config: &Config, mut output: W) -> fmt::Result {
    write_params_and_switches::<visitor::Validate, _>(config, &mut output)?;
    writeln!(output)?;
    writeln!(output, "            Ok(super::Config {{")?;
    write_params_and_switches::<visitor::ConstructConfig, _>(config, &mut output)?;
    writeln!(output, "            }})")?;
    Ok(())
}

fn underscore_to_hypen<W: Write>(mut output: W, ident: &str) -> fmt::Result {
    for c in ident.chars() {
        if c == '_' {
                write!(output, "-")?;
        } else {
                write!(output, "{}", c)?;
        }
    }
    Ok(())
}

fn gen_merge_env<W: Write>(config: &Config, mut output: W) -> fmt::Result {
    for param in &config.params {
        if !param.env_var {
            continue;
        }
        write!(output, "        if let Some(val) = ::std::env::var_os(\"")?;
        config.general.env_prefix.as_ref().map(|prefix| { upper_case(&mut output, &prefix)?; write!(output, "_") }).unwrap_or(Ok(()))?;
        writeln!(output, "{}\") {{", param.name.as_upper_case())?;
        writeln!(output, "            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::Field{})?;", param.name.as_pascal_case())?;
        if let Some(merge_fn) = &param.merge_fn {
            writeln!(output, "            if let Some({}_old) = &mut self.{} {{", param.name.as_snake_case(), param.name.as_snake_case())?;
            writeln!(output, "                {}({}_old, val);", merge_fn, param.name.as_snake_case())?;
            writeln!(output, "            }} else {{")?;
            writeln!(output, "                self.{} = Some(val);", param.name.as_snake_case())?;
            writeln!(output, "            }}")?;
        } else {
            writeln!(output, "            self.{} = Some(val);", param.name.as_snake_case())?;
        }
        writeln!(output, "        }}")?;
    }
    for switch in &config.switches {
        if !switch.env_var {
            continue;
        }
        write!(output, "        if let Some(val) = ::std::env::var_os(\"")?;
        config.general.env_prefix.as_ref().map(|prefix| { upper_case(&mut output, &prefix)?; write!(output, "_") }).unwrap_or(Ok(()))?;
        writeln!(output, "{}\") {{", switch.name.as_upper_case())?;
        if switch.is_count() {
            write!(output, "            let val= <u32 as ::configure_me::parse_arg::ParseArg>::parse_owned_arg(val).map_err(super::EnvParseError::Field{})?;", switch.name.as_pascal_case())?;
            writeln!(output, "            self.{} = Some(val);", switch.name.as_snake_case())?;
        } else {
            writeln!(output, "            if val == *\"1\" || val == *\"true\" {{")?;
            writeln!(output, "                self.{} = Some(true);", switch.name.as_snake_case())?;
            writeln!(output, "            }} else if val == *\"0\" || val == *\"false\" {{")?;
            writeln!(output, "                self.{} = Some(false);", switch.name.as_snake_case())?;
            writeln!(output, "            }} else {{")?;
            writeln!(output, "                return Err(super::EnvParseError::Field{}(val).into());", switch.name.as_pascal_case())?;
            writeln!(output, "            }}")?;
        }
        writeln!(output, "        }}")?;
    }
    Ok(())
}

#[cfg(test)]
fn gen_merge_args<W: Write>(config: &Config, mut output: W) -> fmt::Result {
    write_config::<visitor::MergeArgs, _>(config, &mut output)
}

pub fn generate_code<W: Write>(config: &Config, mut output: W) -> fmt::Result {
    writeln!(output, "pub mod prelude {{")?;
    writeln!(output, "    pub use super::{{Config, ResultExt}};")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "pub enum ArgParseError {{")?;
    writeln!(output, "    MissingArgument(&'static str),")?;
    writeln!(output, "    UnknownArgument(String),")?;
    writeln!(output, "    HelpRequested(String),")?;
    writeln!(output)?;
    gen_arg_parse_error(config, &mut output)?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Display for ArgParseError {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        match self {{")?;
    writeln!(output, "            ArgParseError::MissingArgument(arg) => write!(f, \"A value to argument '{{}}' is missing.\", arg),")?;
    writeln!(output, "            ArgParseError::UnknownArgument(arg) => write!(f, \"An unknown argument '{{}}' was specified.\", arg),")?;
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
    writeln!(output, "pub enum EnvParseError {{")?;
    gen_env_parse_error(config, &mut output)?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Display for EnvParseError {{")?;
    writeln!(output, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{")?;
    writeln!(output, "        match *self {{")?;
    gen_display_env_parse_error(config, &mut output)?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl ::std::fmt::Debug for EnvParseError {{")?;
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
    writeln!(output, "    ConfigParsing {{ file: ::std::path::PathBuf, error: ::configure_me::toml::de::Error }},")?;
    writeln!(output, "    Arguments(ArgParseError),")?;
    writeln!(output, "    Environment(EnvParseError),")?;
    writeln!(output, "    Validation(ValidationError),")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl From<ArgParseError> for Error {{")?;
    writeln!(output, "    fn from(err: ArgParseError) -> Self {{")?;
    writeln!(output, "        Error::Arguments(err)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl From<EnvParseError> for Error {{")?;
    writeln!(output, "    fn from(err: EnvParseError) -> Self {{")?;
    writeln!(output, "        Error::Environment(err)")?;
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
    writeln!(output, "            Error::Environment(err) => write!(f, \"{{}}\", err),")?;
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
    writeln!(output, "    #[serde(crate = \"crate::configure_me::serde\")]")?;
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
    writeln!(output, "            ::configure_me::toml::from_slice(&config_content).map_err(|error| super::Error::ConfigParsing {{ file: config_file_name.as_ref().into(), error }})")?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        pub fn validate(self) -> Result<super::Config, ValidationError> {{")?;
    gen_validation_fn(config, &mut output)?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        pub fn merge_in(&mut self, other: Self) {{")?;
    write_params_and_switches::<visitor::MergeIn, _>(config, &mut output)?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        pub fn merge_args<I: IntoIterator<Item=::std::ffi::OsString>>(&mut self, args: I) -> Result<impl Iterator<Item=::std::ffi::OsString>, super::Error> {{")?;
    writeln!(output, "            let mut iter = args.into_iter().fuse();")?;
    writeln!(output, "            self._program_path = iter.next().map(Into::into);")?;
    writeln!(output)?;
    writeln!(output, "            while let Some(arg) = iter.next() {{")?;
    writeln!(output, "                if arg == *\"--\" {{")?;
    writeln!(output, "                    return Ok(None.into_iter().chain(iter));")?;
    writeln!(output, "                }} else if (arg == *\"--help\") || (arg == *\"-h\") {{")?;
    writeln!(output, "                    return Err(ArgParseError::HelpRequested(self._program_path.as_ref().unwrap().to_string_lossy().into()).into());")?;
    write_config::<visitor::MergeArgs, _>(config, &mut output)?;
    writeln!(output, "                }} else if let Some(mut shorts) = ::configure_me::parse_arg::iter_short(&arg) {{")?;
    writeln!(output, "                    for short in &mut shorts {{")?;
    writeln!(output, "                        if short == 'h' {{")?;
    writeln!(output, "                            return Err(ArgParseError::HelpRequested(self._program_path.as_ref().unwrap().to_string_lossy().into()).into())")?;
    write_config::<visitor::MergeShort, _>(config, &mut output)?;
    writeln!(output, "                        }} else {{")?;
    writeln!(output, "                            let mut arg = String::with_capacity(2);")?;
    writeln!(output, "                            arg.push('-');")?;
    writeln!(output, "                            arg.push(short);")?;
    writeln!(output, "                            return Err(ArgParseError::UnknownArgument(arg).into());")?;
    writeln!(output, "                        }}")?;
    writeln!(output, "                    }}")?;
    writeln!(output, "                }} else if arg.to_str().unwrap_or(\"\").starts_with(\"--\") {{")?;
    writeln!(output, "                    return Err(ArgParseError::UnknownArgument(arg.into_string().unwrap()).into());")?;
    writeln!(output, "                }} else {{")?;
    writeln!(output, "                    return Ok(Some(arg).into_iter().chain(iter))")?;
    writeln!(output, "                }}")?;
    writeln!(output, "            }}")?;
    writeln!(output)?;
    writeln!(output, "            Ok(None.into_iter().chain(iter))")?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        pub fn merge_env(&mut self) -> Result<(), super::Error> {{")?;
    gen_merge_env(config, &mut output)?;
    writeln!(output, "            Ok(())")?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "/// Configuration of the application")?;
    writeln!(output, "pub struct Config {{")?;
    write_params_and_switches::<visitor::ConfigFinal, _>(config, &mut output)?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl Config {{")?;
    writeln!(output, "    pub fn including_optional_config_files<I>(config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>), Error> where I: IntoIterator, I::Item: AsRef<::std::path::Path> {{")?;
    writeln!(output, "        Self::custom_args_and_optional_files(::std::env::args_os(), config_files)")?;
    writeln!(output, "    }}")?;
    writeln!(output)?;
    writeln!(output, "    pub fn custom_args_and_optional_files<A, I>(args: A, config_files: I) -> Result<(Self, impl Iterator<Item=::std::ffi::OsString>), Error> where")?;
    writeln!(output, "        A: IntoIterator, A::Item: Into<::std::ffi::OsString>,")?;
    writeln!(output, "        I: IntoIterator, I::Item: AsRef<::std::path::Path> {{")?;
    writeln!(output)?;
    writeln!(output, "        let mut config = raw::Config::default();")?;
    writeln!(output, "        for path in config_files {{")?;
    writeln!(output, "            match raw::Config::load(path) {{")?;
    writeln!(output, "                Ok(mut new_config) => {{")?;
    writeln!(output, "                    std::mem::swap(&mut config, &mut new_config);")?;
    writeln!(output, "                    config.merge_in(new_config)")?;
    writeln!(output, "                }},")?;
    writeln!(output, "                Err(Error::Reading {{ ref error, .. }}) if error.kind() == ::std::io::ErrorKind::NotFound => (),")?;
    writeln!(output, "                Err(err) => return Err(err),")?;
    writeln!(output, "            }}")?;
    writeln!(output, "        }}")?;
    writeln!(output)?;
    writeln!(output, "        config.merge_env()?;")?;
    writeln!(output, "        let remaining_args = config.merge_args(args.into_iter().map(Into::into))?;")?;
    writeln!(output)?;
    writeln!(output, "        config")?;
    writeln!(output, "            .validate()")?;
    writeln!(output, "            .map(|cfg| (cfg, remaining_args))")?;
    writeln!(output, "            .map_err(Into::into)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "pub trait ResultExt {{")?;
    writeln!(output, "    type Item;")?;
    writeln!(output)?;
    writeln!(output, "    fn unwrap_or_exit(self) -> Self::Item;")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "impl<T> ResultExt for Result<T, Error> {{")?;
    writeln!(output, "    type Item = T;")?;
    writeln!(output)?;
    writeln!(output, "    fn unwrap_or_exit(self) -> Self::Item {{")?;
    writeln!(output, "        use std::io::Write;")?;
    writeln!(output)?;
    writeln!(output, "        match self {{")?;
    writeln!(output, "            Ok(item) => item,")?;
    writeln!(output, "            Err(err @ Error::Arguments(ArgParseError::HelpRequested(_))) => {{")?;
    writeln!(output, "                println!(\"{{}}\", err);")?;
    writeln!(output, "                std::io::stdout().flush().expect(\"failed to flush stdout\");")?;
    writeln!(output, "                ::std::process::exit(0)")?;
    writeln!(output, "            }},")?;
    writeln!(output, "            Err(err) => {{")?;
    writeln!(output, "                eprintln!(\"Error: {{}}\", err);")?;
    writeln!(output, "                std::io::stderr().flush().expect(\"failed to flush stderr\");")?;
    writeln!(output, "                ::std::process::exit(1)")?;
    writeln!(output, "            }}")?;
    writeln!(output, "        }}")?;
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
            let mut out = String::new();
            super::$fn($config, &mut out).unwrap();
            assert_eq!(out, $expected);
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

    #[test]
    fn short_switches_raw_config() {
        check!(gen_raw_config, &config_from(::tests::SHORT_SWITCHES), ::tests::EXPECTED_SHORT_SWITCHES.raw_config);
    }

    #[test]
    fn short_switches_merge_args() {
        check!(gen_merge_args, &config_from(::tests::SHORT_SWITCHES), ::tests::EXPECTED_SHORT_SWITCHES.merge_args);
    }
}
