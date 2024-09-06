set PROXY_URL=localhost:8888/config/rustmo/default/main/rustmo.toml
set JWT_HOST=localhost:8083
cargo run