                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--d", &arg, &mut iter) {
                    let d = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--d"), ArgParseError::FieldD))?;

                    self.d = Some(d);
                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--e", &arg, &mut iter) {
                    let e = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--e"), ArgParseError::FieldE))?;

                    self.e = Some(e);
                } else if arg == *"--a" {
                    self.a = Some(true);
                } else if arg == *"--b" {
                    self.b = Some(true);
                } else if arg == *"--c" {
                    *(self.c.get_or_insert(0)) += 1;
