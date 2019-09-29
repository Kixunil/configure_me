use ::config::Config;
use std::path::PathBuf;
use std::io;
use std::fmt;
use std::convert::{TryFrom, TryInto};
use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        })
    }
}


impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            x => Err(serde::de::Error::unknown_variant(x, &["low", "medium", "high", "critical"])),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DebConfig {
    pub package_name: String,
    #[serde(default)]
    pub config: ConfigMode,
    #[serde(default)]
    pub postinst: PostinstMode,
}

#[derive(Debug, Clone)]
pub enum UmaskError {
    InvalidDigit(u8),
    InvalidLen(usize),
}

impl fmt::Display for UmaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UmaskError::InvalidDigit(digit) => write!(f, "Invalid digit for umask: {:02X}", digit),
            UmaskError::InvalidLen(len) => write!(f, "Invalid umask length: {}, must be 3 or 4", len),
        }
    }
}


#[derive(Debug, Copy, Clone)]
struct UmaskDigit(u8);

impl fmt::Display for UmaskDigit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}


impl TryFrom<u8> for UmaskDigit {
    type Error = UmaskError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        if val >= b'0' && val <= b'9' {
            Ok(UmaskDigit(val - b'0'))
        } else {
            Err(UmaskError::InvalidDigit(val))
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(try_from = "String")]
pub struct Umask {
    special: UmaskDigit,
    user: UmaskDigit,
    group: UmaskDigit,
    others: UmaskDigit,
}

impl Umask {
    pub fn paranoid() -> Self {
        Umask {
            special: UmaskDigit(0),
            user: UmaskDigit(0),
            group: UmaskDigit(7),
            others: UmaskDigit(7),
        }
    }
}

impl fmt::Display for Umask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}{}", self.special, self.user, self.group, self.others)
    }
}


impl TryFrom<String> for Umask {
    type Error = UmaskError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        let bytes = val.as_bytes();

        match TryInto::<&[u8; 3]>::try_into(bytes) {
            Ok(bytes) => Ok(Umask {
                special: UmaskDigit(0),
                user: bytes[0].try_into()?,
                group: bytes[1].try_into()?,
                others: bytes[2].try_into()?,
            }),
            Err(_) => match TryInto::<&[u8; 4]>::try_into(bytes) {
                Ok(bytes) => Ok(Umask {
                    special: bytes[0].try_into()?,
                    user: bytes[1].try_into()?,
                    group: bytes[2].try_into()?,
                    others: bytes[3].try_into()?,
                }),
                Err(_) => Err(UmaskError::InvalidLen(bytes.len()))
            }
        }
    }
}

/// How config file should be generated
#[derive(Debug)]
pub enum ConfigMode {
    /// Generates full script with incldes and `db_go`
    WithBoilerplate,
    /// Creates only "library" which needs to be sourced from the config script
    Lib,
}

impl Default for ConfigMode {
    fn default() -> Self {
        ConfigMode::WithBoilerplate
    }
}

impl<'de> Deserialize<'de> for ConfigMode {
    fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "full" => Ok(ConfigMode::WithBoilerplate),
            "lib" => Ok(ConfigMode::Lib),
            x => Err(serde::de::Error::unknown_variant(x, &["full", "lib"])),
        }
    }
}

/// How postinst file should be generated
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum PostinstMode {
    /// Generates full script which stores the configuration into the specified conf file
    WithBoilerplate { 
        /// Where the configuration of the program will be stored
        conf_file: String,
        /// umask for config file. 0077 by default.
        conf_umask: Option<Umask>,
        /// Owner of the configuration file
        conf_owner: Option<String>,
        /// Group of the configuration file
        conf_group: Option<String>,
    },
    /// Creates only "library" which needs to be sourced from the postinst script
    Lib,
}

impl Default for PostinstMode {
    fn default() -> Self {
        PostinstMode::Lib
    }
}


mod visitor {
    use std::fmt;
    use super::DebConfig;

    pub trait VisitWrite<T> {
        fn visit_write<W: fmt::Write>(&self, config: &DebConfig, output: W) -> fmt::Result;
    }

    impl<'a, T, U> VisitWrite<T> for &'a U where U: VisitWrite<T> {
        fn visit_write<W: fmt::Write>(&self, config: &DebConfig, output: W) -> fmt::Result {
            (*self).visit_write(config, output)
        }
    }

    pub fn iter<T, I, W: fmt::Write>(iter: I, config: &DebConfig, mut output: W) -> fmt::Result where I: IntoIterator, I::Item: VisitWrite<T> {
        for item in iter {
            item.visit_write(config, &mut output)?;
        }
        Ok(())
    }

    pub enum Templates {}
    pub enum Config {}
    pub enum Postinst {}
}

use self::visitor::VisitWrite;

impl VisitWrite<visitor::Templates> for ::config::Config {
    fn visit_write<W: fmt::Write>(&self, deb_config: &DebConfig, mut output: W) -> fmt::Result {
        visitor::iter::<visitor::Templates, _, _>(&self.params, deb_config, &mut output)?;
        visitor::iter::<visitor::Templates, _, _>(&self.switches, deb_config, &mut output)
    }
}

impl VisitWrite<visitor::Config> for ::config::Config {
    fn visit_write<W: fmt::Write>(&self, deb_config: &DebConfig, mut output: W) -> fmt::Result {
        if let ConfigMode::WithBoilerplate = deb_config.config {
            writeln!(output, "#!/bin/bash")?;
            writeln!(output)?;
            writeln!(output, ". /usr/share/debconf/confmodule")?;
            writeln!(output)?;
            visitor::iter::<visitor::Config, _, _>(&self.params, deb_config, &mut output)?;
            visitor::iter::<visitor::Config, _, _>(&self.switches, deb_config, &mut output)?;
            writeln!(output)?;
            writeln!(output, "db_go")
        } else {
            visitor::iter::<visitor::Config, _, _>(&self.params, deb_config, &mut output)?;
            visitor::iter::<visitor::Config, _, _>(&self.switches, deb_config, &mut output)
        }
    }
}

impl VisitWrite<visitor::Postinst> for ::config::Config {
    fn visit_write<W: fmt::Write>(&self, deb_config: &DebConfig, mut output: W) -> fmt::Result {
        if let PostinstMode::WithBoilerplate { conf_file, conf_umask, conf_owner, conf_group, } = &deb_config.postinst {
            writeln!(output, "#!/bin/bash")?;
            writeln!(output)?;
            writeln!(output, ". /usr/share/debconf/confmodule")?;
            writeln!(output)?;
            writeln!(output, "CONF_FILE=\"{}\"", conf_file)?;
            writeln!(output, "OLD_UMASK=`umask`")?;
            writeln!(output, "umask {}", conf_umask.unwrap_or(Umask::paranoid()))?;
            writeln!(output, "echo '# Autogenereated - do NOT modify!' > \"$CONF_FILE\"")?;
            writeln!(output, "echo '# Use dpkg-reconfigure to change the configuration' >> \"$CONF_FILE\"")?;
            if let Some(owner) = conf_owner {
                writeln!(output, "chown {} \"$CONF_FILE\"", owner)?;
            }
            if let Some(group) = conf_group {
                writeln!(output, "chgrp {} \"$CONF_FILE\"", group)?;
            }
            visitor::iter::<visitor::Postinst, _, _>(&self.params, deb_config, &mut output)?;
            visitor::iter::<visitor::Postinst, _, _>(&self.switches, deb_config, &mut output)?;
            writeln!(output, "umask \"$OLD_UMASK\"")?;
            writeln!(output)?;
            writeln!(output, "#DEBHELPER#")
        } else {
            visitor::iter::<visitor::Postinst, _, _>(&self.params, deb_config, &mut output)?;
            visitor::iter::<visitor::Postinst, _, _>(&self.switches, deb_config, &mut output)
        }
    }
}

impl VisitWrite<visitor::Templates> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, config: &DebConfig, mut output: W) -> fmt::Result {
        if self.debconf_priority.is_some() {
            writeln!(output, "Template: {}/{}", config.package_name, self.name.as_snake_case())?;
            if self.ty == "bool" {
                writeln!(output, "Type: bool")?;
            } else {
                writeln!(output, "Type: string")?;
            }
            if let Some(default) = &self.debconf_default {
                writeln!(output, "Default: {}", default)?;
            }
            writeln!(output, "Description: {}", self.doc.as_ref().unwrap_or_else(|| panic!("Parameter {} is missing documentation", self.name.as_snake_case())))?;
            writeln!(output)
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::Templates> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, config: &DebConfig, mut output: W) -> fmt::Result {
        if self.debconf_priority.is_some() {
            writeln!(output, "Template: {}/{}", config.package_name, self.name.as_snake_case())?;
            if self.is_count() {
                writeln!(output, "Type: string")?;
                writeln!(output, "Default: 0")?;
            } else {
                writeln!(output, "Type: bool")?;
                if self.is_inverted() {
                    writeln!(output, "Default: true")?;
                } else {
                    writeln!(output, "Default: false")?;
                }
            }
            writeln!(output, "Description: {}", self.doc.as_ref().unwrap_or_else(|| panic!("Parameter {} is missing documentation", self.name.as_snake_case())))?;
            writeln!(output)
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::Config> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, config: &DebConfig, mut output: W) -> fmt::Result {
        if let Some(priority) = self.debconf_priority {
            writeln!(output, "db_input {} {}/{} || true", priority, config.package_name, self.name.as_snake_case())
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::Config> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, config: &DebConfig, mut output: W) -> fmt::Result {
        if let Some(priority) = self.debconf_priority {
            writeln!(output, "db_input {} {}/{} || true", priority, config.package_name, self.name.as_snake_case())
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::Postinst> for ::config::Param {
    fn visit_write<W: fmt::Write>(&self, config: &DebConfig, mut output: W) -> fmt::Result {
        if self.debconf_priority.is_some() {
            writeln!(output, "db_get {}/{}", config.package_name, self.name.as_snake_case())?;
            match self.ty.as_str() {
                "bool" | "u8" | "u16" | "u32" | "u64" | "u128" |
                    "i8" | "i16" | "i32" | "i64" | "i128" => writeln!(output, "echo {}=\"$RET\" >> \"$CONF_FILE\"", self.name.as_snake_case()),
                _ => writeln!(output, "echo \"$RET\" | sed -e 's/\"/\\\"/g' -e 's/^/{}=\"/' -e 's/$/\"/' >> \"$CONF_FILE\"", self.name.as_snake_case()),
            }
        } else {
            Ok(())
        }
    }
}

impl VisitWrite<visitor::Postinst> for ::config::Switch {
    fn visit_write<W: fmt::Write>(&self, config: &DebConfig, mut output: W) -> fmt::Result {
        if self.debconf_priority.is_some() {
            writeln!(output, "db_get {}/{}", config.package_name, self.name.as_snake_case())?;
            writeln!(output, "echo {}=\"$RET\" >> \"$CONF_FILE\"", self.name.as_snake_case())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Open(io::Error, PathBuf),
    Write(io::Error, PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Open(err, path) => write!(f, "Failed to open file {}: {}", path.display(), err),
            Error::Write(err, path) => write!(f, "Failed to write file {}: {}", path.display(), err),
        }
    }
}


fn write_file<T>(config: &Config, deb_config: &DebConfig, path: PathBuf) -> Result<(), Error> where Config: VisitWrite<T> {
    use std::fs::File;

    let file = File::create(&path);
    match file {
        Ok(file) => ::fmt2io::write(file, |file| VisitWrite::<T>::visit_write(config, deb_config, file)).map_err(|err| Error::Write(err, path)),
        Err(err) => Err(Error::Open(err, path)),
    }
}

pub fn generate_if_requested(config: &Config) -> Result<bool, Error> {
    println!("cargo:rerun-if-env-changed=DEBCONF_OUT");
    let debconf_out = std::env::var_os("DEBCONF_OUT").map(PathBuf::from);
    if let (Some(out_dir), Some(debconf)) = (&debconf_out, &config.debconf) {

        write_file::<visitor::Templates>(config, debconf, out_dir.join("templates"))?;
        write_file::<visitor::Config>(config, debconf, out_dir.join("config"))?;
        write_file::<visitor::Postinst>(config, debconf, out_dir.join("postinst"))?;

        Ok(true)
    } else {
        Ok(false)
    }
}
