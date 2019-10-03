            let foo = self.foo;
            let bar = self.bar;

            Ok(super::Config {
                foo: foo.map(Into::into),
                bar: bar.map(Into::into),
            })
