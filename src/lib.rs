#![cfg(target_arch = "wasm32")]

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
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use utils::{EXPECTED_DATA, set_panic_hook};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::js_sys::Uint8Array;
pub use wasm_bindgen_rayon::init_thread_pool;

mod utils;

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
    base_url: Option<String>,
    fetcher: Option<js_sys::Function>,
    client: reqwest::Client,
    cache: Arc<Mutex<HashMap<u8, midnight_transient_crypto::proofs::ParamsProver>>>,
}

#[wasm_bindgen]
impl MidnightWasmParamsProvider {
    pub fn new(base_url: String) -> Self {
        Self::with_source(Some(base_url), None)
    }

    #[wasm_bindgen(js_name = "newWithFetcher")]
    pub fn new_with_fetcher(fetcher: js_sys::Function) -> Self {
        Self::with_source(None, Some(fetcher))
    }
}

impl MidnightWasmParamsProvider {
    fn with_source(base_url: Option<String>, fetcher: Option<js_sys::Function>) -> Self {
        Self {
            base_url,
            fetcher,
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl midnight_transient_crypto::proofs::ParamsProverProvider for MidnightWasmParamsProvider {
    async fn get_params(
        &self,
        k: u8,
    ) -> std::io::Result<midnight_transient_crypto::proofs::ParamsProver> {
        if let Some(cached) = self.cache.lock().unwrap().get(&k).cloned() {
            return Ok(cached);
        }

        let data = EXPECTED_DATA[k as usize - 10];
        let raw = self.fetch_params_bytes(k, data.0).await?;

        if raw.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Prover params not found or empty. Expected file {}", data.0),
            ));
        }

        let mut hasher = sha2::Sha256::new();

        hasher.update(&raw);

        let hash = <[u8; 32]>::from(hasher.finalize());

        if hash != data.1 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Hash mismatch for k: {k}. The params file may be outdated or corrupted."),
            ))
        } else {
            let params = midnight_transient_crypto::proofs::ParamsProver::read(Cursor::new(raw))
                .map_err(|_e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Can't deserialize prover params".to_string(),
                    )
                })?;
            self.cache.lock().unwrap().insert(k, params.clone());
            Ok(params)
        }
    }
}

impl MidnightWasmParamsProvider {
    async fn fetch_params_bytes(&self, k: u8, expected_file: &str) -> std::io::Result<Vec<u8>> {
        if let Some(fetcher) = &self.fetcher {
            let result = fetcher
                .call1(&JsValue::NULL, &JsValue::from(k))
                .map_err(|err| {
                    js_error_to_io(
                        err,
                        std::io::ErrorKind::Other,
                        "Params fetcher threw before returning",
                    )
                })?;

            let promise = js_sys::Promise::resolve(&result);
            let value = JsFuture::from(promise).await.map_err(|err| {
                js_error_to_io(
                    err,
                    std::io::ErrorKind::Other,
                    "Params fetcher promise rejected",
                )
            })?;

            return js_value_to_bytes(
                value,
                std::io::ErrorKind::InvalidData,
                "Params fetcher must return a Uint8Array or ArrayBuffer",
            );
        }

        let base_url = self.base_url.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No base URL or custom params fetcher configured",
            )
        })?;

        let url = join_url(base_url, expected_file)?;

        self.client
            .get(url.clone())
            .send()
            .await
            .map_err(|_e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to fetch data from {url}"),
                )
            })?
            .bytes()
            .await
            .map_err(|_e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Expected response to be bytes".to_string(),
                )
            })
            .map(|bytes| bytes.to_vec())
    }
}

fn js_error_to_io(err: JsValue, kind: std::io::ErrorKind, context: &str) -> std::io::Error {
    let message = err
        .as_string()
        .or_else(|| {
            js_sys::JSON::stringify(&err)
                .ok()
                .and_then(|s| s.as_string())
        })
        .unwrap_or_else(|| "unknown JS error".to_string());

    std::io::Error::new(kind, format!("{context}: {message}"))
}

fn js_value_to_bytes(
    value: JsValue,
    kind: std::io::ErrorKind,
    context: &str,
) -> std::io::Result<Vec<u8>> {
    match value.dyn_into::<js_sys::Uint8Array>() {
        Ok(bytes) => Ok(bytes.to_vec()),
        Err(value) => match value.dyn_into::<js_sys::ArrayBuffer>() {
            Ok(buffer) => Ok(Uint8Array::new(&buffer).to_vec()),
            Err(_) => Err(std::io::Error::new(kind, context.to_string())),
        },
    }
}

fn join_url(base_url: &str, path: &str) -> std::io::Result<String> {
    let base = url::Url::parse(base_url).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid base URL {base_url}: {e}"),
        )
    })?;

    base.join(path).map(|url| url.to_string()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Failed to join URL path {path}: {e}"),
        )
    })
}

#[derive(Clone, Copy)]
enum ResolverArtifactType {
    ProverKey,
    VerifierKey,
    IrSource,
}

impl ResolverArtifactType {
    fn path_component(self) -> &'static str {
        match self {
            Self::ProverKey => "pk",
            Self::VerifierKey => "vk",
            Self::IrSource => "ir",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::ProverKey => "prover key",
            Self::VerifierKey => "verifier key",
            Self::IrSource => "IR source",
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmResolver {
    base_url: Option<String>,
    prover_key_fetcher: Option<js_sys::Function>,
    verifier_key_fetcher: Option<js_sys::Function>,
    ir_source_fetcher: Option<js_sys::Function>,
    client: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, ProvingKeyMaterial>>>,
}

#[wasm_bindgen]
impl WasmResolver {
    pub fn new(base_url: String) -> Self {
        Self::with_sources(Some(base_url), None, None, None)
    }

    #[wasm_bindgen(js_name = "newWithFetchers")]
    pub fn new_with_fetchers(
        prover_key_fetcher: js_sys::Function,
        verifier_key_fetcher: js_sys::Function,
        ir_source_fetcher: js_sys::Function,
    ) -> Self {
        Self::with_sources(
            None,
            Some(prover_key_fetcher),
            Some(verifier_key_fetcher),
            Some(ir_source_fetcher),
        )
    }
}

impl WasmResolver {
    fn with_sources(
        base_url: Option<String>,
        prover_key_fetcher: Option<js_sys::Function>,
        verifier_key_fetcher: Option<js_sys::Function>,
        ir_source_fetcher: Option<js_sys::Function>,
    ) -> Self {
        Self {
            base_url,
            prover_key_fetcher,
            verifier_key_fetcher,
            ir_source_fetcher,
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
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

        if let Some(cached) = self.cache.lock().unwrap().get(key_path).cloned() {
            return Ok(Some(cached));
        }

        let pk_raw = self
            .fetch_key_data(key_path, ResolverArtifactType::ProverKey)
            .await?;
        let vk_raw = self
            .fetch_key_data(key_path, ResolverArtifactType::VerifierKey)
            .await?;
        let ir_raw = self
            .fetch_key_data(key_path, ResolverArtifactType::IrSource)
            .await?;

        let proving_key_material = ProvingKeyMaterial {
            prover_key: pk_raw,
            verifier_key: vk_raw,
            ir_source: ir_raw,
        };
        self.cache
            .lock()
            .unwrap()
            .insert(key_path.to_string(), proving_key_material.clone());

        Ok(Some(proving_key_material))
    }
}

impl WasmResolver {
    async fn fetch_key_data(
        &self,
        key_path: &str,
        artifact_type: ResolverArtifactType,
    ) -> std::io::Result<Vec<u8>> {
        let fetcher = match artifact_type {
            ResolverArtifactType::ProverKey => self.prover_key_fetcher.as_ref(),
            ResolverArtifactType::VerifierKey => self.verifier_key_fetcher.as_ref(),
            ResolverArtifactType::IrSource => self.ir_source_fetcher.as_ref(),
        };

        if let Some(fetcher) = fetcher {
            let result = fetcher
                .call1(&JsValue::NULL, &JsValue::from(key_path))
                .map_err(|err| {
                    js_error_to_io(
                        err,
                        std::io::ErrorKind::Other,
                        "Resolver fetcher threw before returning",
                    )
                })?;

            let promise = js_sys::Promise::resolve(&result);
            let value = JsFuture::from(promise).await.map_err(|err| {
                js_error_to_io(
                    err,
                    std::io::ErrorKind::Other,
                    "Resolver fetcher promise rejected",
                )
            })?;

            let raw = js_value_to_bytes(
                value,
                std::io::ErrorKind::InvalidData,
                "Resolver fetcher must return a Uint8Array or ArrayBuffer",
            )?;
            if raw.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!(
                        "{} not found or empty for {key_path}/{}",
                        artifact_type.label(),
                        artifact_type.path_component()
                    ),
                ));
            }

            return Ok(raw);
        }

        let base_url = self.base_url.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No base URL or custom resolver fetchers configured",
            )
        })?;

        let url = join_url(
            base_url,
            &format!("{key_path}/{}", artifact_type.path_component()),
        )?;

        let raw = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|_e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to fetch {} from {url}", artifact_type.label()),
                )
            })?
            .bytes()
            .await
            .map_err(|_e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Expected {} response to be bytes", artifact_type.label()),
                )
            })?;

        if raw.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("{} not found or empty at {url}", artifact_type.label()),
            ));
        }

        Ok(raw.to_vec())
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
