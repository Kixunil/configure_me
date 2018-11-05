                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = ::configure_me::parse_arg::ParseArg::parse_owned_arg(foo)
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
                } else if arg == *"--bar" {
                    let bar = iter.next().ok_or(ArgParseError::MissingArgument("--bar"))?;

                    let bar = ::configure_me::parse_arg::ParseArg::parse_owned_arg(bar)
                        .map_err(ArgParseError::FieldBar)?;

                    self.bar = Some(bar);
                } else if arg == *"--baz" {
                    let baz = iter.next().ok_or(ArgParseError::MissingArgument("--baz"))?;

                    let baz = ::configure_me::parse_arg::ParseArg::parse_owned_arg(baz)
                        .map_err(ArgParseError::FieldBaz)?;

                    self.baz = Some(baz);
                } else if arg == *"--verbose" {
                    self.verbose = Some(true);
                } else if arg == *"--no-fast" {
                    self.fast = Some(false);
