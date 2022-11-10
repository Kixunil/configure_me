#[cfg(feature = "codespan-reporting")]
#[test]
fn codespan_report() {
    let toml = r#"
        [general]
        conf_file_param = "foo"
        conf_dir_param = "foo"
        skip_default_conf_files_switch = "help"

        [[param]]
        name = "bar"
        type = "bool"
        default = "false"
        abbr = "x"
        optional = false

        [[switch]]
        name = "foo"
        abbr = "x"

        [[switch]]
        name = "foo"
        abbr = "x"

        [[switch]]
        name = "baz"
        abbr = "-"

        [[switch]]
        name = "0foo"

        [[switch]]
        name = "-foo"

        [[switch]]
        name = "foo-bar"

        [[switch]]
        name = "1a=bit**loong   and ###weird@parameter"

        [[switch]]
        name = "-a=bit**loong   and ###weird@parameter"

        [[switch]]
        name = "a=bit**loong   and ###weird@parameter"
        "#;

    let expected = r#"invalid config specification:
error: the option `foo` appears more than once
   ┌─ unknown file:4:26
   │
 3 │         conf_file_param = "foo"
   │                           ----- the option was first defined here
 4 │         conf_dir_param = "foo"
   │                          ^^^^^ the option is repeated here
   ·
15 │         name = "foo"
   │                ^^^^^ ... and here
   ·
19 │         name = "foo"
   │                ^^^^^ ... and here

error: use of reserved option
  ┌─ unknown file:5:42
  │
5 │         skip_default_conf_files_switch = "help"
  │                                          ^^^^^^ this option is reserved because it's always implemented by `configure_me`

error: parameter attempts to be both optional and mandatory at the same time
   ┌─ unknown file:10:19
   │
 8 │         name = "bar"
   │                ----- in the parameter `bar`
 9 │         type = "bool"
10 │         default = "false"
   │                   ^^^^^^^ the default value is provided here making the parameter optional
11 │         abbr = "x"
12 │         optional = false
   │                    ^^^^^ setting `optional` to `false` makes the parameter mandatory here
   │
   = Help: either make the parameter optional or remove the default value

error: the option `x` appears more than once
   ┌─ unknown file:16:16
   │
11 │         abbr = "x"
   │                --- the option was first defined here
   ·
16 │         abbr = "x"
   │                ^^^ the option is repeated here
   ·
20 │         abbr = "x"
   │                ^^^ ... and here

error: invalid short option
   ┌─ unknown file:24:16
   │
23 │         name = "baz"
   │                ----- in this field
24 │         abbr = "-"
   │                ^^^ this option uses an invalid character
   │
   = Note: only English letters (both lower case and upper case) are allowed

error: the identifier `0foo` contains an invalid character
   ┌─ unknown file:27:17
   │
27 │         name = "0foo"
   │                 ^ the identifier starts with a digit
   │
   = Help: identifiers mut not start with digits

error: the identifier `-foo` contains an invalid character
   ┌─ unknown file:30:17
   │
30 │         name = "-foo"
   │                 ^ this char is invalid
   │
   = Help: dashes are prepended automatically, you don't need to write them

error: the identifier `foo-bar` contains an invalid character
   ┌─ unknown file:33:20
   │
33 │         name = "foo-bar"
   │                    ^ this char is invalid
   │
   = Help: consider replacing dashes with underscores.
           They will be replaced with dashes in command line parameters
           but stay underscores in config files.

error: the identifier `1a=bit**loong   and ###weird@parameter` contains invalid characters
   ┌─ unknown file:36:17
   │
36 │         name = "1a=bit**loong   and ###weird@parameter"
   │                 ^ ^   ^^     ^^^   ^^^^     ^ ... and this char
   │                 │ │   │      │     │         
   │                 │ │   │      │     ... and these chars
   │                 │ │   │      ... and these chars
   │                 │ │   ... and these chars
   │                 │ this char is invalid
   │                 the identifier starts with a digit
   │
   = Help: identifiers mut not start with digits

error: the identifier `-a=bit**loong   and ###weird@parameter` contains invalid characters
   ┌─ unknown file:39:17
   │
39 │         name = "-a=bit**loong   and ###weird@parameter"
   │                 ^ ^   ^^     ^^^   ^^^^     ^ ... and this char
   │                 │ │   │      │     │         
   │                 │ │   │      │     ... and these chars
   │                 │ │   │      ... and these chars
   │                 │ │   ... and these chars
   │                 │ ... and this char
   │                 this char is invalid
   │
   = Help: dashes are prepended automatically, you don't need to write them

error: the identifier `a=bit**loong   and ###weird@parameter` contains invalid characters
   ┌─ unknown file:42:18
   │
42 │         name = "a=bit**loong   and ###weird@parameter"
   │                  ^   ^^     ^^^   ^^^^     ^ ... and this char
   │                  │   │      │     │         
   │                  │   │      │     ... and these chars
   │                  │   │      ... and these chars
   │                  │   ... and these chars
   │                  this char is invalid

"#;
    let error_messages = format!("{:?}", configure_me_codegen::generate_source(toml.as_bytes(), std::io::sink()).unwrap_err());
    assert_eq!(error_messages, expected);
}
