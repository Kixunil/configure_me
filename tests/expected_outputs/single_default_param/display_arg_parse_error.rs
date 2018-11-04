        ArgParseError::FieldFoo(err) => write!(f, "Failed to parse argument '--foo': {}.", err),
