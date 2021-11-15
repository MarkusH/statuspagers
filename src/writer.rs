use std::fs::{copy, create_dir_all, read_dir, File};
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

pub fn render(tera: &Tera, context: &Context, template_file: &str, output_file: PathBuf) {
    eprintln!("Writing file {} ...", output_file.to_str().unwrap());
    let f = File::create(output_file).unwrap();
    let b = BufWriter::new(f);
    tera.render_to(template_file, context, b).unwrap();
    eprintln!("    Done!");
}

// https://stackoverflow.com/a/65192210
pub fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    eprintln!(
        "Copying directory {} to {} ...",
        src.as_ref().to_str().unwrap(),
        dst.as_ref().to_str().unwrap()
    );
    create_dir_all(&dst)?;
    for entry in read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            eprintln!("--> {}", entry.path().to_str().unwrap());
            copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
