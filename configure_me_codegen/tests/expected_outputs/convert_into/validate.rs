            let foo = self.foo.unwrap_or_else(|| { 42 });
            let bar = self.bar;
            let baz = self.baz.ok_or(ValidationError::MissingField("baz"))?;

            Ok(super::Config {
                foo,
                bar: bar.map(Into::into),
                baz: baz.into(),
            })
