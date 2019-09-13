        ArgParseError::HelpRequested(program_name) => write!(f, "Usage: {} [-d D|--d D] [-e E|--e E] [-a|--a] [-b|--b] [-c|--c ...] [-f|--foo-bar]\n\nArguments:\n        -a, --a    test", program_name),
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
