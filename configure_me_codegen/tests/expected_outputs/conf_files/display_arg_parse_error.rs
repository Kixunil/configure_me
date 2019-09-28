        ArgParseError::HelpRequested(program_name) => write!(f, "Usage: {} [--config CONF_FILE] [--conf-dir CONF_DIR] [--foo FOO]\n\nArguments:\n        --config      Load configuration from this file.\n        --conf-dir    Load configuration from files in this directory.\n        --foo         A foo", program_name),
        ArgParseError::FieldFoo(err) => {
            write!(f, "Failed to parse argument '--foo': {}.\n\nHint: the value must be ", err)?;
            <u32 as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
        ArgParseError::OpenConfDir(err, dir) => write!(f, "Failed to open configuration directory {}: {}", dir.display(), err),
        ArgParseError::ReadConfDir(err, dir) => write!(f, "Failed to read configuration directory {}: {}", dir.display(), err),
