        if let Some(val) = ::std::env::var_os("TEST_APP_FOO") {
            if val == *"1" || val == *"true" {
                self.foo = Some(true);
            } else if val == *"0" || val == *"false" {
                self.foo = Some(false);
            } else {
                return Err(super::EnvParseError::FieldFoo(val).into());
            }
        }
