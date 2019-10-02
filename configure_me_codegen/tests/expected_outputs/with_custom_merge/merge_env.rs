        if let Some(val) = ::std::env::var_os("TEST_APP_FOO") {
            let val = ::configure_me::parse_arg::ParseArg::parse_owned_arg(val).map_err(super::EnvParseError::FieldFoo)?;
            if let Some(foo_old) = &mut self.foo {
                (|a: &mut u32, b: u32| *a += b)(foo_old, val);
            } else {
                self.foo = Some(val);
            }
        }
