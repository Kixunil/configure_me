                        } else if short == 'd' {
                            self.d = Some(shorts.parse_remaining(&mut iter).map_err(|err| err.map_or(ArgParseError::MissingArgument("-d"), ArgParseError::FieldD))?);
                            break;
                        } else if short == 'e' {
                            self.e = Some(shorts.parse_remaining(&mut iter).map_err(|err| err.map_or(ArgParseError::MissingArgument("-e"), ArgParseError::FieldE))?);
                            break;
                        } else if short == 'a' {
                            self.a = Some(true);
                        } else if short == 'b' {
                            self.b = Some(true);
                        } else if short == 'c' {
                            *(self.c.get_or_insert(0)) += 1;
