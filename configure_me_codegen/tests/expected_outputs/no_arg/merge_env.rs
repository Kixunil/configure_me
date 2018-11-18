        if let Some(val) = ::std::env::var_os("TEST_APP_FOO") {
            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::FieldFoo)?;
            self.foo = Some(val);
        }
