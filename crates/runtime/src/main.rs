use std::path::PathBuf;

use clap::{Parser, Subcommand};
use runtime::{
    Runtime,
    prototype::Prototype,
    task::{Event, Task},
};
use workflow::Workflow;

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

impl Commands {
    fn workflow(&self) -> &PathBuf {
        match self {
            Commands::Parse { workflow } => workflow,
            Commands::Run { workflow } => workflow,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let mut runtime = Runtime::new()?;
    let workflow = Workflow::load(args.command.workflow())?;
    let prototype = Prototype::new(&mut runtime, &workflow).await?;

    if let Commands::Run { .. } = args.command {
        let mut task = Task::new(&mut runtime, &prototype).await?;
        let mut subscribe = task.subscribe();

        tokio::spawn(async move {
            task.run().await;
        });

        while let Ok(event) = subscribe.recv().await {
            match event {
                Event::ExecutionStarted(node_id, params) => {
                    println!("{node_id} started with params: {params:?}")
                }
                Event::ExecutionSucceeded(node_id, output) => {
                    println!("{node_id} succeeded with output: {output:?}")
                }
                Event::ExecutionFailed(node_id, error) => {
                    eprintln!("{node_id} failed with error: {error:?}")
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Prototype(#[from] runtime::prototype::Error),
    #[error(transparent)]
    Runtime(#[from] runtime::Error),
    #[error(transparent)]
    Task(#[from] runtime::task::Error),
    #[error(transparent)]
    Workflow(#[from] workflow::Error),
}
