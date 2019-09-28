fn process_template(test_name: &str, out_dir: &str) {
    use std::io::{self, Write, BufRead, BufReader, BufWriter};

    let file = std::fs::File::open("tests/expected_outputs/config-template.rs").unwrap();
    let file = BufReader::new(file);
    let mut output = BufWriter::new(std::fs::File::create(format!("{}/{}-config.rs", out_dir, test_name)).unwrap());
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
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = format!("{}/expected_outputs", out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();

    let tests = ["empty", "single_optional_param", "single_mandatory_param", "single_default_param", "single_switch", "multiple_params", "no_arg", "short_switches", "conf_files"];

    for test in &tests {
        process_template(test, &out_dir);
    }
}
