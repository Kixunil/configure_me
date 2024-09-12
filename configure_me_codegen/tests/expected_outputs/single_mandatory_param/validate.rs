            let foo = self.foo.ok_or(ValidationError::MissingField("foo"))?;

            #[allow(clippy::useless_conversion)]
            Ok(super::Config {
                foo: foo.into(),
            })
