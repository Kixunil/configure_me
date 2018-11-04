        ArgParseError::FieldFoo(err) => write!(f, "Failed to parse argument '--foo': {}.", err),
        ArgParseError::FieldBar(err) => write!(f, "Failed to parse argument '--bar': {}.", err),
        ArgParseError::FieldBaz(err) => write!(f, "Failed to parse argument '--baz': {}.", err),
