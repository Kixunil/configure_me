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
        if let Some(val) = ::std::env::var_os("TEST_APP_VERBOSE") {
            if val == *"1" || val == *"true" {
                self.verbose = Some(true);
            } else if val == *"0" || val == *"false" {
                self.verbose = Some(false);
            } else {
                return Err(super::EnvParseError::FieldVerbose(val).into());
            }
        }
        if let Some(val) = ::std::env::var_os("TEST_APP_FAST") {
            if val == *"1" || val == *"true" {
                self.fast = Some(true);
            } else if val == *"0" || val == *"false" {
                self.fast = Some(false);
            } else {
                return Err(super::EnvParseError::FieldFast(val).into());
            }
        }
