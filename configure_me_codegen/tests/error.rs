#[cfg(feature = "codespan-reporting")]
#[test]
fn codespan_report() {
    let toml = r#"
        [general]
        conf_file_param = "foo"
        conf_dir_param = "foo"
        skip_default_conf_files_switch = "help"
        program_name = "foo"

        [param.bar]
        type = "bool"
        default = "false"
        abbr = "x"
        optional = false

        [param.foo]
        type = "String"

        [switch.foo]
        abbr = "x"

        [switch.baz]
        abbr = "-"

        [switch.0foo]

        [switch.-foo]

        [switch.foo-bar]

        [switch."1a=bit**loong   and ###weird@parameter"]

        [switch."-a=bit**loong   and ###weird@parameter"]

        [switch."a=bit**loong   and ###weird@parameter"]
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
14 │         [param.foo]
   │                ^^^ ... and here
   ·
17 │         [switch.foo]
   │                 ^^^ ... and here

error: use of reserved option
  ┌─ unknown file:5:42
  │
5 │         skip_default_conf_files_switch = "help"
  │                                          ^^^^^^ this option is reserved because it's always implemented by `configure_me`

error: `foo` is not a valid program name handling strategy
  ┌─ unknown file:6:24
  │
6 │         program_name = "foo"
  │                        ^^^^^ this is an invalid program name handling strategy
  │
  = Help: valid program name handling strategies are `unused`, `optional`, `required`.

error: parameter attempts to be both optional and mandatory at the same time
   ┌─ unknown file:10:19
   │
 8 │         [param.bar]
   │                --- in the parameter `bar`
 9 │         type = "bool"
10 │         default = "false"
   │                   ^^^^^^^ the default value is provided here making the parameter optional
11 │         abbr = "x"
12 │         optional = false
   │                    ^^^^^ setting `optional` to `false` makes the parameter mandatory here
   │
   = Help: either make the parameter optional or remove the default value

error: the option `x` appears more than once
   ┌─ unknown file:18:16
   │
11 │         abbr = "x"
   │                --- the option was first defined here
   ·
18 │         abbr = "x"
   │                ^^^ the option is repeated here

error: invalid short option
   ┌─ unknown file:21:16
   │
20 │         [switch.baz]
   │                 --- in this field
21 │         abbr = "-"
   │                ^^^ this option uses an invalid character
   │
   = Note: only English letters (both lower case and upper case) are allowed

error: the identifier `0foo` contains an invalid character
   ┌─ unknown file:23:17
   │
23 │         [switch.0foo]
   │                 ^ the identifier starts with a digit
   │
   = Help: identifiers mut not start with digits

error: the identifier `-foo` contains an invalid character
   ┌─ unknown file:25:17
   │
25 │         [switch.-foo]
   │                 ^ this char is invalid
   │
   = Help: dashes are prepended automatically, you don't need to write them

error: the identifier `foo-bar` contains an invalid character
   ┌─ unknown file:27:20
   │
27 │         [switch.foo-bar]
   │                    ^ this char is invalid
   │
   = Help: consider replacing dashes with underscores.
           They will be replaced with dashes in command line parameters
           but stay underscores in config files.

error: the identifier `1a=bit**loong   and ###weird@parameter` contains invalid characters
   ┌─ unknown file:29:18
   │
29 │         [switch."1a=bit**loong   and ###weird@parameter"]
   │                  ^ ^   ^^     ^^^   ^^^^     ^ ... and this char
   │                  │ │   │      │     │         
   │                  │ │   │      │     ... and these chars
   │                  │ │   │      ... and these chars
   │                  │ │   ... and these chars
   │                  │ this char is invalid
   │                  the identifier starts with a digit
   │
   = Help: identifiers mut not start with digits

error: the identifier `-a=bit**loong   and ###weird@parameter` contains invalid characters
   ┌─ unknown file:31:18
   │
31 │         [switch."-a=bit**loong   and ###weird@parameter"]
   │                  ^ ^   ^^     ^^^   ^^^^     ^ ... and this char
   │                  │ │   │      │     │         
   │                  │ │   │      │     ... and these chars
   │                  │ │   │      ... and these chars
   │                  │ │   ... and these chars
   │                  │ ... and this char
   │                  this char is invalid
   │
   = Help: dashes are prepended automatically, you don't need to write them

error: the identifier `a=bit**loong   and ###weird@parameter` contains invalid characters
   ┌─ unknown file:33:19
   │
33 │         [switch."a=bit**loong   and ###weird@parameter"]
   │                   ^   ^^     ^^^   ^^^^     ^ ... and this char
   │                   │   │      │     │         
   │                   │   │      │     ... and these chars
   │                   │   │      ... and these chars
   │                   │   ... and these chars
   │                   this char is invalid

"#;
    let error_messages = format!("{:?}", configure_me_codegen::generate_source(toml.as_bytes(), std::io::sink()).unwrap_err());
    assert_eq!(error_messages, expected);
}
