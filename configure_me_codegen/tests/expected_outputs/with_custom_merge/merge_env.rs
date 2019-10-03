        if let Some(val) = ::std::env::var_os("TEST_APP_FOO") {
            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::FieldFoo)?;
            if let Some(foo_old) = &mut self.foo {
                (|a: &mut u32, b: u32| *a += b)(foo_old, val);
            } else {
                self.foo = Some(val);
            }
        }
        if let Some(val) = ::std::env::var_os("TEST_APP_BAR") {
            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::FieldBar)?;
            if let Some(bar_old) = &mut self.bar {
                (|a: &mut String, b: String| a.push_str(&b))(bar_old, val);
            } else {
                self.bar = Some(val);
            }
        }
