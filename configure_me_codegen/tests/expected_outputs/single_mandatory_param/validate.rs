            let foo = self.foo.ok_or(ValidationError::MissingField("foo"))?;

            Ok(super::Config {
                foo: foo.into(),
            })
