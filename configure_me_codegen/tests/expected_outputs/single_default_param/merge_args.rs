                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--foo", &arg, &mut iter) {
                    let foo = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--foo"), ArgParseError::FieldFoo))?;

                    self.foo = Some(foo);
