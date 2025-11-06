use ledger_storage::db::InMemoryDB;
use midnight_base_crypto::signatures::Signature;
use midnight_ledger::structure::{ProofPreimageMarker, Transaction};
use midnight_onchain_runtime::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    proofs::{KeyLocation, ProvingKeyMaterial},
};
use rand::SeedableRng as _;
use rand::rngs::StdRng;
use sha2::Digest as _;
use std::io::Cursor;
use utils::{EXPECTED_DATA, set_panic_hook};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;
pub use wasm_bindgen_rayon::init_thread_pool;

mod utils;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    set_panic_hook();
    Ok(())
}

#[wasm_bindgen]
pub struct WasmProver {
    pp: MidnightWasmParamsProvider,
    resolver: WasmResolver,
}

#[wasm_bindgen]
impl WasmProver {
    #[allow(clippy::new_without_default)]
    pub fn new(resolver: WasmResolver, pp: MidnightWasmParamsProvider) -> WasmProver {
        WasmProver { pp, resolver }
    }

    pub async fn prove(
        &self,
        rng: &Rng,
        unproven_tx: &Uint8Array,
        cost_model: &CostModel,
    ) -> Result<Uint8Array, JsError> {
        let tx: Transaction<Signature, ProofPreimageMarker, PedersenRandomness, InMemoryDB> =
            tagged_deserialize(&unproven_tx.to_vec()[..])
                .map_err(|e| JsError::new(e.to_string().as_ref()))?;

        let provider = zkir::LocalProvingProvider {
            rng: rng.0.clone(),
            params: &self.pp,
            resolver: &self.resolver,
        };

        let unbalanced_tx = tx
            .prove(provider, &cost_model.0)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;

        let mut res = Vec::new();
        tagged_serialize(&unbalanced_tx, &mut res)?;
        Ok(Uint8Array::from(&res[..]))
    }
}

#[wasm_bindgen]
pub struct CostModel(midnight_onchain_runtime::cost_model::CostModel);

#[wasm_bindgen]
impl CostModel {
    pub fn deserialize(bytes: &Uint8Array) -> Result<Self, JsError> {
        let cost_model = tagged_deserialize(&bytes.to_vec()[..])
            .map_err(|e| JsError::new(e.to_string().as_ref()))?;

        Ok(Self(cost_model))
    }

    #[wasm_bindgen(js_name = "initialCostModel")]
    pub fn initial_cost_model() -> Self {
        Self(INITIAL_COST_MODEL)
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct MidnightWasmParamsProvider {
    base_url: String,
}

#[wasm_bindgen]
impl MidnightWasmParamsProvider {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl midnight_transient_crypto::proofs::ParamsProverProvider for MidnightWasmParamsProvider {
    async fn get_params(
        &self,
        k: u8,
    ) -> std::io::Result<midnight_transient_crypto::proofs::ParamsProver> {
        let data = EXPECTED_DATA[k as usize - 10];

        let mut url = self.base_url.clone();
        url.push('/');
        url.push_str(data.0);

        let raw = reqwest::Client::new()
            .get(url.clone())
            .send()
            .await
            .map_err(|_e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to fetch data from {url}"),
                )
            })?
            .bytes()
            .await
            .map_err(|_e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expected response to be bytes".to_string(),
                )
            })?;

        if raw.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Prover params not found or empty. Expected file {} at {}",
                    data.0, url
                ),
            ));
        }

        let mut hasher = sha2::Sha256::new();

        hasher.update(&raw);

        let hash = <[u8; 32]>::from(hasher.finalize());

        if hash != data.1 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Hash mismatch for k: {k}. This means the file may be outdated or corrupted. This may be fixing by clearing the cache."
                ),
            ))
        } else {
            midnight_transient_crypto::proofs::ParamsProver::read(Cursor::new(raw)).map_err(|_e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Can't deserialize prover params".to_string(),
                )
            })
        }
    }
}

#[wasm_bindgen]
pub struct WasmResolver {
    base_url: String,
    client: reqwest::Client,
}

#[wasm_bindgen]
impl WasmResolver {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

impl midnight_transient_crypto::proofs::Resolver for WasmResolver {
    async fn resolve_key(&self, key: KeyLocation) -> std::io::Result<Option<ProvingKeyMaterial>> {
        let key_path = if key.0.starts_with('/') {
            &key.0[1..]
        } else {
            &key.0
        };

        let fetch_key_data = |suffix: String, key_type: String| async move {
            let url = format!("{}/{}/{}", self.base_url, key_path, suffix);

            let raw = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|_e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to fetch {key_type} from {url}"),
                    )
                })?
                .bytes()
                .await
                .map_err(|_e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Expected {key_type} response to be bytes"),
                    )
                })?;

            if raw.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{key_type} not found or empty at {url}"),
                ));
            }

            Ok(raw.to_vec())
        };

        let pk_raw = fetch_key_data("pk".to_string(), "prover key".to_string()).await?;
        let vk_raw = fetch_key_data("vk".to_string(), "verifier key".to_string()).await?;
        let ir_raw = fetch_key_data("ir".to_string(), "IR source".to_string()).await?;

        Ok(Some(ProvingKeyMaterial {
            prover_key: pk_raw,
            verifier_key: vk_raw,
            ir_source: ir_raw,
        }))
    }
}

#[wasm_bindgen]
pub struct Rng(StdRng);

#[wasm_bindgen]
impl Rng {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Rng(StdRng::from_rng(rand::thread_rng()).expect("couldn't initialize Rng"))
    }
}
