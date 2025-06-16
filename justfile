check:
    cargo +nightly fmt
    cargo check
    cargo clippy --allow-dirty --fix
    cargo machete
    cargo sort-derives

install: 
    cargo install --locked cargo-component
    cargo install --locked cargo-machete
    cargo install --locked cargo-sort-derives
    cargo install --locked cargo-watch
    cargo install --locked systemfd
    cargo install --locked wkg
    pnpm install

parse workflow:
    WASMTIME_BACKTRACE_DETAILS=1 cargo run -p runtime -- parse -w ./examples/{{workflow}}.json

run workflow:
    WASMTIME_BACKTRACE_DETAILS=1 cargo run -p runtime -- run -w ./examples/{{workflow}}.json
