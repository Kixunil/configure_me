        if let Some(val) = ::std::env::var_os("TEST_APP_FOO") {
            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::FieldFoo)?;
            self.foo = Some(val);
        }
        if let Some(val) = ::std::env::var_os("TEST_APP_BAR") {
            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::FieldBar)?;
            self.bar = Some(val);
        }
        if let Some(val) = ::std::env::var_os("TEST_APP_BAZ") {
            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::FieldBaz)?;
            self.baz = Some(val);
        }
