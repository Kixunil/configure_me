            #[allow(clippy::unnecessary_lazy_evaluations)]
            let foo = self.foo.unwrap_or_else(|| { 42 });

            #[allow(clippy::useless_conversion)]
            Ok(super::Config {
                foo: foo.into(),
            })
