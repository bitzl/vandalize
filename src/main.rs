use clap::{App, AppSettings, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use rand::{thread_rng, Rng};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let matches = App::new("Vandalize!")
        .author("Marcus Bitzl <marcus@bitzl.io>")
        .about("Randomize bytes in files")
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp])
        .subcommand(
            App::new("random")
                .about("randomize random bytes")
                .arg_from_usage("<input> 'The file to create vandalized copies from.'")
                .arg_from_usage("<output> 'The directory to put the vandalized copies.'")
                .arg(
                    Arg::from_usage("-n, --number 'Number of vandalized copies to create.'")
                        .default_value("1"),
                ),
        )
        .subcommand(
            App::new("every")
                .about("creates a randomized copy for every byte")
                .arg_from_usage("<input> 'The file to create vandalized copies from.'")
                .arg_from_usage("<output> 'The directory to put the vandalized copies.'")
                .arg(Arg::from_usage("-e --every 'Glitch every n-th byte.'").default_value("1")),
        )
        .get_matches();

    let input = Path::new(matches.value_of("INPUT").unwrap());

    let source = Source::new(input);

    let target = Path::new(matches.value_of("TARGET").unwrap());

    match matches.subcommand_name() {
        Some("random") => {
            let number = usize::from_str_radix(matches.value_of("number").unwrap(), 10).unwrap();
            vandalize(input, target, number)
        }
        Some("every") => {
            let every_nth = usize::from_str_radix(matches.value_of("every").unwrap(), 10).unwrap();
            // let resolution = usize::from_str_radix(matches.value_of("number").unwrap(), 10).unwrap();
            every(&source, target, every_nth)
        }
        Some(other) => println!("Unknown subcommand {}", other),
        None => println!("Must provide subcommand"),
    }
}

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

    vandalize_bytes(data, &random_byte_indices, buffer);
}

struct Source {
    base_name: String,
    extension: String,
    data: Vec<u8>,
}

impl Source {
    fn new(path: &Path) -> Source {
        let base_name: String = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let extension: String = path.extension().unwrap().to_str().unwrap().to_owned();
        let data = fs::read(path).unwrap();
        Source {
            base_name,
            extension,
            data,
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

fn every(source: &Source, target_dir: &Path, every_nth: usize) {
    let total = source.len() / every_nth;
    let digits = total.to_string().len();

    let pb = ProgressBar::new(total as u64);
    pb.set_style(ProgressStyle::default_bar().template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec} | {eta})",
    ));

    for i in 0..total {
        let byte_index = i * every_nth;
        let filename = format!(
            "{}_v{:0digits$}.{}",
            source.base_name,
            byte_index,
            source.extension,
            digits = digits
        );
        let target = target_dir.join(filename);

        let mut writer: BufWriter<_> = BufWriter::new(File::create(target).unwrap());

        vandalize_bytes(&source.data, &vec![byte_index], &mut writer);
        pb.tick();
    }
    pb.finish();
}

fn vandalize_bytes<W: Write>(data: &[u8], indices: &[usize], buffer: &mut W) {
    let mut rng = thread_rng();

    let mut last_end: usize = 0;
    for index in indices {
        buffer.write_all(&data[last_end..*index]).unwrap();
        let random_byte: Vec<u8> = vec![rng.gen()];
        buffer.write_all(&random_byte).unwrap();

        last_end = index + 1;
    }

    if last_end < data.len() {
        buffer.write_all(&data[last_end..data.len()]).unwrap();
    }
}
