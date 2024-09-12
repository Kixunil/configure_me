            #[allow(clippy::unnecessary_lazy_evaluations)]
            let foo = self.foo.unwrap_or_else(|| { 42 });
            let bar = self.bar;
            let baz = self.baz.ok_or(ValidationError::MissingField("baz"))?;

            #[allow(clippy::useless_conversion)]
            Ok(super::Config {
                foo: foo.into(),
                bar: bar.map(Into::into),
                baz: baz.into(),
                verbose: self.verbose.unwrap_or(false),
                fast: self.fast.unwrap_or(true),
            })
