                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--foo", &arg, &mut iter) {
                    let foo = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--foo"), ArgParseError::FieldFoo))?;

                    self.foo = Some(foo);
                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--bar", &arg, &mut iter) {
                    let bar = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--bar"), ArgParseError::FieldBar))?;

                    self.bar = Some(bar);
                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--baz", &arg, &mut iter) {
                    let baz = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--baz"), ArgParseError::FieldBaz))?;

                    self.baz = Some(baz);
                } else if arg == *"--verbose" {
                    self.verbose = Some(true);
                } else if arg == *"--no-fast" {
                    self.fast = Some(false);
