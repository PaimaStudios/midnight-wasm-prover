import init, {
  initThreadPool,
  MidnightWasmParamsProvider,
  Rng,
  WasmProver,
  WasmResolver,
  CostModel,
} from "@paima/midnight-wasm-prover";
import type { ProverMessage, ProverResponse } from "./worker-types.js";

// @ts-ignore
await init();

await initThreadPool(navigator.hardwareConcurrency);

self.postMessage({
  type: "wasm-ready",
  message: "worker pool initialized initialized",
} as ProverResponse);

let prover: WasmProver | undefined;
let rng = Rng.new();

async function runProver(serializedTx: Uint8Array) {
  try {
    const startTime = performance.now();
    console.time("prove");
    const result = await prover!.prove(rng, serializedTx, CostModel.initialCostModel());
    console.timeEnd("prove");
    const endTime = performance.now();
    const durationMs = Math.round(endTime - startTime);

    console.log(`proven raw tx: ${uint8ArrayToHex(result)}`);

    self.postMessage({
      type: "success",
      data: result,
      durationMs: durationMs,
    } as ProverResponse);
  } catch (error) {
    console.log("error on prove function u.u");
    self.postMessage({
      type: "error",
      message: error instanceof Error ? error.message : String(error),
    } as ProverResponse);
  }
}

function uint8ArrayToHex(uint8Array: Uint8Array) {
  return Array.from(uint8Array, function(byte) {
    return ("0" + (byte & 0xff).toString(16)).slice(-2);
  }).join("");
}

self.onmessage = async (event: MessageEvent<ProverMessage>) => {
  console.log("onmessage received:", event.data);
  const { type } = event.data;

  if (type === "params") {
    const { baseUrl } = event.data;
    console.log("initializing params with baseUrl:", baseUrl);

    const resolver = WasmResolver.new(baseUrl);
    const pp = MidnightWasmParamsProvider.new(baseUrl);

    prover = WasmProver.new(resolver, pp);

    self.postMessage({ type: "params-ready" });
  } else if (type === "prove") {
    const { serializedTx } = event.data;
    await runProver(serializedTx);
  }
};
