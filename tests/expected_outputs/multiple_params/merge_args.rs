                } else if arg == *"--foo" {
                    let foo = iter.next().ok_or(ArgParseError::MissingArgument("--foo"))?;

                    let foo = foo
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--foo"))?
                        .parse()
                        .map_err(ArgParseError::FieldFoo)?;

                    self.foo = Some(foo);
                } else if arg == *"--bar" {
                    let bar = iter.next().ok_or(ArgParseError::MissingArgument("--bar"))?;

                    let bar = bar
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--bar"))?
                        .parse()
                        .map_err(ArgParseError::FieldBar)?;

                    self.bar = Some(bar);
                } else if arg == *"--baz" {
                    let baz = iter.next().ok_or(ArgParseError::MissingArgument("--baz"))?;

                    let baz = baz
                        .to_str()
                        .ok_or(ArgParseError::BadUtf8("--baz"))?
                        .parse()
                        .map_err(ArgParseError::FieldBaz)?;

                    self.baz = Some(baz);
                } else if arg == *"--verbose" {
                    self.verbose = Some(true);
                } else if arg == *"--no-fast" {
                    self.fast = Some(false);
