        ArgParseError::HelpRequested(program_name) => write!(f, "Usage: {} [--foo FOO]", program_name),
        ArgParseError::FieldFoo(err) => write!(f, "Failed to parse argument '--foo': {}.", err),
