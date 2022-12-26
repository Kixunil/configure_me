        ArgParseError::HelpRequested(program_name) => write!(f, "Usage: {} [--foo FOO] [--bar BAR] [--baz BAZ]\n\nArguments:\n        --foo    A foo\n        --bar    A very, very, very, very, very, very, very, very, very, very, \n                 very, very, very, very long documentation...\n        --baz    A much, much, much, much, much, much, much, much, much, much, \n                 much, much, much, much, much, much, much, much, much, much, \n                 much, much, much, much, much, much, much, much, much, much, \n                 much, much, much, much, much, much, much, much, much, much, \n                 much, much longer documentation...", program_name),
        ArgParseError::FieldFoo(err) => {
            write!(f, "Failed to parse argument '--foo': {}.\n\nHint: the value must be ", err)?;
            <u32 as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
        ArgParseError::FieldBar(err) => {
            write!(f, "Failed to parse argument '--bar': {}.\n\nHint: the value must be ", err)?;
            <String as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
        ArgParseError::FieldBaz(err) => {
            write!(f, "Failed to parse argument '--baz': {}.\n\nHint: the value must be ", err)?;
            <String as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
