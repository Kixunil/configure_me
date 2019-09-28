                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--config", &arg, &mut iter) {
                    let file_path: std::path::PathBuf = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--config"), |never| match never {}))?;
                    let config = Config::load(file_path)?;
                    self.merge_in(config);
                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--conf-dir", &arg, &mut iter) {
                    let dir_path: std::path::PathBuf = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--conf-dir"), |never| match never {}))?;

                    let dir = match std::fs::read_dir(&dir_path) {
                        Ok(dir) => dir,
                        Err(err) => return Err(ArgParseError::OpenConfDir(err, dir_path).into()),
                    };

                    for file in dir {
                        let file = match file {
                            Ok(file) => file,
                            Err(err) => return Err(ArgParseError::ReadConfDir(err, dir_path).into()),
                        };

                        let config = Config::load(file.path())?;
                        self.merge_in(config);
                    }
                } else if let Some(value) = ::configure_me::parse_arg::match_arg("--foo", &arg, &mut iter) {
                    let foo = value.map_err(|err| err.map_or(ArgParseError::MissingArgument("--foo"), ArgParseError::FieldFoo))?;

                    self.foo = Some(foo);
