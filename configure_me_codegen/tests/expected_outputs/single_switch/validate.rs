
            #[allow(clippy::useless_conversion)]
            Ok(super::Config {
                foo: self.foo.unwrap_or(false),
            })
