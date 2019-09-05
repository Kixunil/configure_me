        ArgParseError::HelpRequested(program_name) => write!(f, "Usage: {} [--d D] [--e E] [--a] [--b] [--c]", program_name),
        ArgParseError::FieldD(err) => {
            write!(f, "Failed to parse argument '--d': {}.\n\nHint: the value must be ", err)?;
            <String as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
        ArgParseError::FieldE(err) => {
            write!(f, "Failed to parse argument '--e': {}.\n\nHint: the value must be ", err)?;
            <String as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
