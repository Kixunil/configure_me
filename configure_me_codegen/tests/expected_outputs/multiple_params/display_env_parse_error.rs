        EnvParseError::FieldFoo(ref err) => {
            write!(f, "Failed to parse environment variable 'TEST_APP_FOO': {}.\n\nHint: the value must be ", err)?;
            <u32 as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
        EnvParseError::FieldBar(ref err) => {
            write!(f, "Failed to parse environment variable 'TEST_APP_BAR': {}.\n\nHint: the value must be ", err)?;
            <String as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
        EnvParseError::FieldBaz(ref err) => {
            write!(f, "Failed to parse environment variable 'TEST_APP_BAZ': {}.\n\nHint: the value must be ", err)?;
            <String as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
        EnvParseError::FieldVerbose(ref err) => {
            write!(f, "Invalid value '{:?}' for 'TEST_APP_VERBOSE'.\n\nHint: the allowed values are 0, false, 1, true.", err)
        },
        EnvParseError::FieldFast(ref err) => {
            write!(f, "Invalid value '{:?}' for 'TEST_APP_FAST'.\n\nHint: the allowed values are 0, false, 1, true.", err)
        },
