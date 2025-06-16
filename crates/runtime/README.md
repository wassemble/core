# Flow Engine

A Rust-based workflow engine that executes WebAssembly (Wasm) components in a defined sequence. This engine can be used both as a command-line tool and as a library in your Rust projects.

## Features

- Execute Wasm components using wasmtime
- Define workflows using a simple JSON format
- Use as a CLI tool or embed as a library
- Flexible node-based workflow execution

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
flow-engine = { path = "./engine" }
```

## Usage

### Command Line Interface

Run a workflow using the CLI:

```bash
cargo run -- --workflow workflow.json
```

### Library Usage

```rust
use flow_engine::Engine;

// Initialize the engine
let engine = Engine::new();

// Load and execute a workflow
engine.load_workflow("workflow.json")?;
engine.execute()?;
```

## Workflow JSON Format

The workflow.json file defines a sequence of nodes that execute Wasm components. Here's the structure:

```json
{
  "nodes": [
    {
      "id": "node1",
      "component": "path/to/component.wasm",
      "function": "function_name",
      "inputs": {
        "param1": "value1",
        "param2": "value2"
      }
    },
    {
      "id": "node2",
      "component": "path/to/another.wasm",
      "function": "another_function",
      "inputs": {
        "param1": "value1"
      },
      "depends_on": ["node1"]
    }
  ]
}
```

### Node Properties

- `id`: Unique identifier for the node
- `component`: Path to the Wasm component file
- `function`: Name of the function to execute in the Wasm component
- `inputs`: Key-value pairs of input parameters for the function
- `depends_on`: (Optional) Array of node IDs that must complete before this node executes

## Example Workflow

Here's a simple example of a workflow that processes data through multiple Wasm components:

```json
{
  "nodes": [
    {
      "id": "load_data",
      "component": "components/data_loader.wasm",
      "function": "load",
      "inputs": {
        "source": "data.json"
      }
    },
    {
      "id": "process_data",
      "component": "components/processor.wasm",
      "function": "process",
      "inputs": {
        "format": "json"
      },
      "depends_on": ["load_data"]
    },
    {
      "id": "save_result",
      "component": "components/saver.wasm",
      "function": "save",
      "inputs": {
        "destination": "output.json"
      },
      "depends_on": ["process_data"]
    }
  ]
}
```

## Building

To build the project:

```bash
cargo build
```

For release build:

```bash
cargo build --release
```

## License

[Add your license information here]
