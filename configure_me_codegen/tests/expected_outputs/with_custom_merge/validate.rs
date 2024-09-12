            let foo = self.foo;
            let bar = self.bar;

            #[allow(clippy::useless_conversion)]
            Ok(super::Config {
                foo: foo.map(Into::into),
                bar: bar.map(Into::into),
            })
