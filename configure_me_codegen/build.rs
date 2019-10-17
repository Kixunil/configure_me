use std::path::{Path, PathBuf};

fn process_template(test_name: &str, out_dir: &Path) {
    use std::io::{self, Write, BufRead, BufReader, BufWriter};

    let file = std::fs::File::open("tests/expected_outputs/config-template.rs").unwrap();
    let file = BufReader::new(file);
    let out_file_name = out_dir.join(format!("{}-config.rs", test_name));
    let output = std::fs::File::create(&out_file_name).expect("Failed to open test output file");
    let mut output = BufWriter::new(output);
    for line in file.lines() {
        let line = line.unwrap();

        if line.starts_with("<<\"") && line.ends_with("\">>") {
            let file_name = format!("tests/expected_outputs/{}/{}", test_name, &line[3..(line.len() - 3)]);
            eprintln!("Reading: {}", file_name);
            let mut src = std::fs::File::open(&file_name).unwrap();
            io::copy(&mut src, &mut output).unwrap();
        } else {
            writeln!(output, "{}", line).unwrap();
        }
    }
}

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("Missing OUT_DIR"));
    let out_dir_expected_outputs = out_dir.join("expected_outputs");
    std::fs::create_dir_all(&out_dir_expected_outputs).unwrap();

    let tests = ["empty", "single_optional_param", "single_mandatory_param", "single_default_param", "single_switch", "multiple_params", "no_arg", "short_switches", "conf_files", "with_custom_merge"];

    for test in &tests {
        process_template(test, &out_dir_expected_outputs);
    }
}
