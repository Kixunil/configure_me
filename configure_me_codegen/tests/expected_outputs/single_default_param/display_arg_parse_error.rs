        ArgParseError::HelpRequested(program_name) => write!(f, "Usage: {} [--foo FOO]", program_name),
        ArgParseError::FieldFoo(err) => {
            write!(f, "Failed to parse argument '--foo': {}.\n\nHint: the value must be ", err)?;
            <u32 as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
