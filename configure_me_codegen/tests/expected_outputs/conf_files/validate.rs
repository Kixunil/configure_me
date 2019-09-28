            let foo = self.foo.unwrap_or_else(|| { 42 });

            Ok(super::Config {
                foo: foo.into(),
            })
