use clap::Parser;
use workflow::Workflow;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    source: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let content = tokio::fs::read_to_string(&cli.source).await?;
    let workflow: Workflow = if cli.source.ends_with(".json") {
        serde_json::from_str(&content)?
    } else if cli.source.ends_with(".yaml") || cli.source.ends_with(".yml") {
        serde_yaml::from_str(&content)?
    } else {
        return Err("File must be either JSON or YAML".into());
    };

    // println!(
    //     "Workflow parsed successfully with {} nodes and {} edges",
    //     workflow.nodes.len(),
    //     workflow.edges.len()
    // );

    Ok(())
}
