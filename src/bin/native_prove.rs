use anyhow::{Context, Result, anyhow};
use ledger_storage::db::InMemoryDB;
use midnight_base_crypto::signatures::Signature;
use midnight_ledger::structure::{ProofMarker, ProofPreimageMarker, Transaction};
use midnight_onchain_runtime::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    proofs::{KeyLocation, ParamsProver, ParamsProverProvider, ProvingKeyMaterial, Resolver},
};
use rand::SeedableRng as _;
use rand::rngs::StdRng;
use sha2::Digest as _;
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Instant;

struct FsParamsProvider {
    root: PathBuf,
}

impl ParamsProverProvider for FsParamsProvider {
    async fn get_params(&self, k: u8) -> std::io::Result<ParamsProver> {
        let path = self.root.join(format!("bls_midnight_2p{k}"));
        let raw = fs::read(&path)?;
        Ok(ParamsProver::read(Cursor::new(raw))?)
    }
}

struct FsResolver {
    root: PathBuf,
}

impl Resolver for FsResolver {
    async fn resolve_key(&self, key: KeyLocation) -> std::io::Result<Option<ProvingKeyMaterial>> {
        let key_path = key.0.trim_start_matches('/');
        let key_root = self.root.join(key_path);

        let prover_key = fs::read(key_root.join("pk"))
            .or_else(|_| fs::read(key_root.with_extension("prover")))?;
        let verifier_key = fs::read(key_root.join("vk"))
            .or_else(|_| fs::read(key_root.with_extension("verifier")))?;
        let ir_source = fs::read(key_root.join("ir"))
            .or_else(|_| fs::read(key_root.with_extension("bzkir")))?;

        Ok(Some(ProvingKeyMaterial {
            prover_key,
            verifier_key,
            ir_source,
        }))
    }
}

fn verify_params_hash(params_root: &Path) -> Result<()> {
    let path = params_root.join("bls_midnight_2p14");
    let raw = fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let hash = <[u8; 32]>::from(sha2::Sha256::digest(&raw));
    let expected =
        const_hex::decode("fc253016885ec830e97808c9ec920bb5cab5c21af590380a6cb5eb0538e2b244")?;

    if hash.as_slice() != expected.as_slice() {
        return Err(anyhow!(
            "unexpected hash for {}: got {}, expected {}",
            path.display(),
            const_hex::encode(hash),
            const_hex::encode(expected)
        ));
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let tx_path = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("usage: cargo run --bin native_prove -- <tx.bin> [params_root] [artifacts_root]"))?;
    let params_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../midnight-ledger/result"));
    let artifacts_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("webpack-demo/public"));

    verify_params_hash(&params_root)?;

    let tx_bytes =
        fs::read(&tx_path).with_context(|| format!("failed to read {}", tx_path.display()))?;
    let tx: Transaction<Signature, ProofPreimageMarker, PedersenRandomness, InMemoryDB> =
        tagged_deserialize(&tx_bytes[..]).context("failed to deserialize unproven transaction")?;

    let resolver = FsResolver {
        root: artifacts_root,
    };
    let params = FsParamsProvider { root: params_root };
    let rng = StdRng::from_rng(rand::thread_rng()).context("failed to initialize RNG")?;

    let prove_started_at = Instant::now();
    let proven: Transaction<Signature, ProofMarker, PedersenRandomness, InMemoryDB> =
        futures::executor::block_on(async {
            tx.prove(
                zkir::LocalProvingProvider {
                    rng,
                    resolver: &resolver,
                    params: &params,
                },
                &INITIAL_COST_MODEL,
            )
            .await
        })
        .context("native proving failed")?;
    let prove_elapsed = prove_started_at.elapsed();

    let mut proven_bytes = Vec::new();
    tagged_serialize(&proven, &mut proven_bytes).context("failed to serialize proven transaction")?;

    println!("input_bytes={}", tx_bytes.len());
    println!("output_bytes={}", proven_bytes.len());
    println!("prove_duration_ms={}", prove_elapsed.as_millis());

    Ok(())
}
