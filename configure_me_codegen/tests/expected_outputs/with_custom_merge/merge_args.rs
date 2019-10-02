                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--foo", &arg, &mut iter) {
                    let foo = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--foo"), ArgParseError::FieldFoo))?;

                    if let Some(foo_old) = &mut self.foo {
                        (|a: &mut u32, b: u32| *a += b)(foo_old, foo);
                    } else {
                        self.foo = Some(foo);
                    }
