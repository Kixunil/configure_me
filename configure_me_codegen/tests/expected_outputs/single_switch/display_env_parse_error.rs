        EnvParseError::FieldFoo(ref err) => {
            write!(f, "Invalid value '{:?}' for 'TEST_APP_FOO'.\n\nHint: the allowed values are 0, false, 1, true.", err)
        },
