use std::{path::PathBuf, pin::pin};

use clap::{Parser, Subcommand};
use engine::{executor::Executor, metadata::Metadata, workflow::Workflow, Engine};
use tokio_stream::StreamExt;

/// A CLI tool for executing workflows
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Parse {
        /// Path to the workflow manifest file
        #[arg(short, long)]
        workflow: PathBuf,
    },
    Run {
        /// Path to the workflow manifest file
        #[arg(short, long)]
        workflow: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    match args.command {
        Commands::Parse { workflow } => {
            let workflow = load_workflow(&workflow)?;
            let mut engine = Engine::new()?;
            let metadata = Metadata::new(&mut engine, workflow.sources()).await;
            println!("{}", serde_json::to_string_pretty(&metadata)?);
        }
        Commands::Run { workflow } => {
            let workflow = load_workflow(&workflow)?;
            let mut engine = Engine::new()?;
            engine.load_components(&workflow.sources()).await?;
            let executor = Executor::new(&workflow)?;
            let stream = executor.run(&mut engine);
            let mut stream = pin!(stream);
            while let Some((node_id, result)) = stream.next().await {
                match result {
                    Ok(outputs) => {
                        println!("Node {node_id:?} output: {outputs:?}");
                    }
                    Err(e) => {
                        println!("Node {node_id:?} error: {e:?}");
                    }
                }
            }
        }
    }

    Ok(())
}

fn load_workflow(path: &PathBuf) -> Result<Workflow, Error> {
    let source = std::fs::read_to_string(path)?;
    let is_json = path.extension().is_some_and(|ext| ext == "json");
    let workflow: Workflow = match is_json {
        true => serde_json::from_str(&source)?,
        false => serde_yaml::from_str(&source)?,
    };
    Ok(workflow)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Engine(#[from] engine::Error),
    #[error(transparent)]
    Executor(#[from] engine::executor::Error),
    #[error(transparent)]
    File(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}
