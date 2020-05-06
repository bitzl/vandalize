use clap::{App, AppSettings, Arg};
use rand::{thread_rng, Rng};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn vandalize(input: &Path, target_dir: &Path, copies: usize) {
    let data = fs::read(input).unwrap();

    let base_name = input.file_stem().unwrap().to_str().unwrap();
    let extension = input.extension().unwrap().to_str().unwrap();
    let width = copies.to_string().len();

    for i in 0..copies {
        let filename = format!("{}_v{:0width$}.{}", base_name, i, extension, width = width);
        let target = target_dir.join(filename);

        let mut writer: BufWriter<_> = BufWriter::new(File::create(target).unwrap());

        vandalize_data(&data, 1, &mut writer);
    }
}

fn vandalize_data<W: Write>(data: &[u8], bytes_to_break: usize, buffer: &mut W) {
    let mut rng = thread_rng();

    let mut random_byte_indices: Vec<usize> = (0..bytes_to_break)
        .map(|_| rng.gen_range(0, data.len()))
        .collect();
    random_byte_indices.sort();

    let mut last_end: usize = 0;
    for index in random_byte_indices {
        buffer.write_all(&data[last_end..index]).unwrap();
        let random_byte: Vec<u8> = vec![rng.gen()];
        buffer.write_all(&random_byte).unwrap();

        last_end = index + 1;
    }

    if last_end < data.len() {
        buffer.write_all(&data[last_end..data.len()]).unwrap();
    }
}

fn main() {
    let matches = App::new("prometheus-sd")
        .author("Marcus Bitzl <marcus@bitzl.io>")
        .about("Randomize bytes in files")
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp])
        .arg(
            Arg::with_name("INPUT")
                .help("The file to create vandalized copies from.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("TARGET")
                .help("The directory to put the vandalized copies.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("number")
                .help("Number of vandalized copies to create.")
                .short("-n")
                .long("--number")
                .default_value("1")
                .required(true),
        )
        .get_matches();

    let input = Path::new(matches.value_of("INPUT").unwrap());
    let target = Path::new(matches.value_of("TARGET").unwrap());
    let number = usize::from_str_radix(matches.value_of("number").unwrap(), 10).unwrap();

    vandalize(input, target, number);
}
