            let foo = self.foo;

            Ok(super::Config {
                foo: foo.map(Into::into),
            })
