# Flow

A full-stack application with a Rust backend and a frontend application.

## Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://rustup.rs) - The Rust programming language and Cargo package manager
- [cargo-component](https://github.com/bytecodealliance/cargo-component) - A Cargo subcommand for building WebAssembly components
- [pnpm](https://pnpm.io) - A fast, disk space efficient package manager for Node.js

## Installation

1. Install Rust:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install cargo-component:

   ```bash
   cargo install cargo-component
   ```

3. Install pnpm:
   ```bash
   npm install -g pnpm
   ```

## Development

The project uses [just](https://github.com/casey/just) as a command runner. Here are the available commands:

### Check Dependencies

To verify all required dependencies are installed:

```bash
just doctor
```

### Start Development Servers

To start both the frontend and backend development servers:

```bash
just dev
```

This will:

- Start the frontend development server (in the `app` directory)
- Start the Rust backend server with hot-reloading enabled

## Project Structure

- `app/` - Frontend application
- `server/` - Rust backend server
