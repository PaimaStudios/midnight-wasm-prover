build:
    RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-arg=--max-memory=4294967296 --cfg getrandom_backend="wasm_js"' \
      rustup run nightly-2025-10-01 \
      wasm-pack build --release --target web . -d pkg \
      -- -Z build-std=panic_abort,std

    jq 'del(.type) | .files += ["snippets"] | .name = "@paima/midnight-wasm-prover"' ./pkg/package.json > ./pkg/tmp.json && mv ./pkg/tmp.json ./pkg/package.json
