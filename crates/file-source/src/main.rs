use clap::Parser;
use file_source::FileSource;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    source: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let source = FileSource::parse(&cli.source)?;
    let content = source.load().await?;
    println!("File size: {} bytes", content.len());

    Ok(())
}
