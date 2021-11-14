use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use tera::{Context, Tera};

pub fn render(tera: &Tera, context: &Context, template_file: &str, output_file: PathBuf) {
    eprintln!("Writing file {} ...", output_file.to_str().unwrap());
    let f = File::create(output_file).unwrap();
    let b = BufWriter::new(f);
    tera.render_to(template_file, context, b).unwrap();
    eprintln!("    Done!");
}
