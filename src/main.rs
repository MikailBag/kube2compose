mod compose;
mod convert;
mod load;

use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
struct Opts {
    #[clap(long, short = "f")]
    file: Vec<PathBuf>,
    #[clap(long, short = "o")]
    output: PathBuf,
}
fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let objects = load::load(&opts.file)?;

    let mut out = compose::Compose {
        ..Default::default()
    };

    let converter = convert::Converter::new(&mut out, &objects);
    converter.convert()?;

    let config = serde_yaml::to_vec(&out)?;
    std::fs::write(&opts.output, config)?;

    Ok(())
}
