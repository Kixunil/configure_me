        EnvParseError::FieldFoo(ref err) => {
            write!(f, "Failed to parse environment variable 'TEST_APP_FOO': {}.\n\nHint: the value must be ", err)?;
            <u32 as ::configure_me::parse_arg::ParseArg>::describe_type(&mut *f)?;
            write!(f, ".")
        },
