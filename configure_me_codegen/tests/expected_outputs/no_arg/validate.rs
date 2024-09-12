            let foo = self.foo;

            #[allow(clippy::useless_conversion)]
            Ok(super::Config {
                foo: foo.map(Into::into),
            })
