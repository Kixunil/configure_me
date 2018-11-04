                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = foo
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--foo"))?
                        .parse()
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
