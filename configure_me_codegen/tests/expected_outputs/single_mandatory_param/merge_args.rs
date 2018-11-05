                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = ::configure_me::parse_arg::ParseArg::parse_owned_arg(foo)
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
