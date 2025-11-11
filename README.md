# Midnight wasm prover bindings and demo

This package contains slightly opinionated bindings for proving
[Midnight](https://midnight.network/) transactions in the browser.

The `webpack-demo` directory can be used as a template. It produces the proof
for a random zswap transaction.

# Building

- Get [rustup](https://rustup.rs/)
- Ensure the nightly toolchain is installed: `rustup toolchain install
nightly-2025-10-01`.
- Get [wasm-pack](https://drager.github.io/wasm-pack/installer/) if needed.
- Get [just](https://github.com/casey/just) and run `just build`. Or manually
run the commands in `justfile`.

*NOTE: * For more details about the flags see the [wasm-bindgen-rayon documentation](https://github.com/RReverser/wasm-bindgen-rayon?tab=readme-ov-file#building-rust-code).

## Demo

For the demo in `webpack-demo`:

Either build the wasm package with the steps above, or get it from the [releases](https://github.com/PaimaStudios/midnight-wasm-prover/releases):

```sh
wget https://github.com/PaimaStudios/midnight-wasm-prover/releases/download/v0.1.0-alpha1/paima-midnight-wasm-prover-0.1.0-alpha1.tgz
tar -xzf paima-midnight-wasm-prover-0.1.0-alpha1.tgz
mv package pkg
```

Then

```sh
cd webpack-demo
npm install
npm run serve
```

## Usage

>  **NOTE:** For the thread pool to work, it's necessary to set the
>  `Cross-Origin-Embedder-Policy` and `Cross-Origin-Opener-Policy` headers.
>
>  For example, for the webpack devserver:
>
>  ```js
>  // webpack.config.js
>
>  module.exports = {
>    devServer: {
>      headers: [
>        {
>          key: 'Cross-Origin-Embedder-Policy',
>          value: 'require-corp',
>        },
>        {
>          key: 'Cross-Origin-Opener-Policy',
>          value: 'same-origin',
>        },
>      ]
>    }
>  }
>  ```

The entry point is the `WasmProver.prove` function. It takes an unproven
serialized transaction and outputs a proven serialized transaction.

The `WasmProver` is instantiated with two resolvers.

`WasmResolver` is used to fetch the proving materials. The final URL is
constructed by concatenating `{baseUrl}/{circuit_key}/{pk|vk|ir}`. The `public`
directory in `webpack-demo` contains an example for the zswap circuit.

`MidnightWasmParamsProvider` is used to fetch the KZG trusted setup parameters.
The url is just `{baseUrl}/{filename}`.

## Notes

- The demo has a `vendor` directory, which contains wasm
packages. These were extracted by running `nix build` in
[midnight-ledger](https://github.com/midnightntwrk/midnight-ledger), and copied
from the `result` directory. They are committed mostly as a temporary convenience
(since the libs are not published) for testing.
- This targets at minimum `ledger-6.1.0`. For previous versions see
`@paima/midnight-vm-bindings`.

## Limitations

- While the prover uses a pool of webworkers, it's also necessary to call the
`prove` function in its own worker in order to not lock the browser thread. This
is the setup also used in the demo.
