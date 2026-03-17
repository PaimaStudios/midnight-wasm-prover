import init, {
  initThreadPool,
  MidnightWasmParamsProvider,
  Rng,
  WasmProver,
  WasmResolver,
  CostModel,
} from "@paima/midnight-wasm-prover";
import type { ProverMessage, ProverResponse } from "./worker-types.js";

self.addEventListener("error", (event) => {
  self.postMessage({
    type: "error",
    message: event.message || "Unhandled worker error",
  } as ProverResponse);
});

self.addEventListener("unhandledrejection", (event) => {
  const reason =
    event.reason instanceof Error ? event.reason.message : String(event.reason);
  self.postMessage({
    type: "error",
    message: `Unhandled worker rejection: ${reason}`,
  } as ProverResponse);
});

self.addEventListener("messageerror", () => {
  self.postMessage({
    type: "error",
    message: "Worker message deserialization failed",
  } as ProverResponse);
});

let prover: WasmProver | undefined;
let rng: Rng | undefined;

async function runProver(serializedTx: Uint8Array) {
  try {
    if (!prover || !rng) {
      throw new Error("Prover worker is not initialized");
    }
    const startTime = performance.now();
    const result = await prover.prove(rng, serializedTx, CostModel.initialCostModel());
    const endTime = performance.now();
    const durationMs = Math.round(endTime - startTime);

    self.postMessage({
      type: "success",
      data: result,
      durationMs: durationMs,
    } as ProverResponse);
  } catch (error) {
    self.postMessage({
      type: "error",
      message: error instanceof Error ? error.message : String(error),
    } as ProverResponse);
  }
}

async function initializeWasm() {
  try {
    // @ts-ignore
    await init();
    rng = Rng.new();
    const threadCount = navigator.hardwareConcurrency;
    await initThreadPool(threadCount);

    self.postMessage({
      type: "wasm-ready",
      message: "worker pool initialized",
    } as ProverResponse);
  } catch (error) {
    self.postMessage({
      type: "error",
      message: error instanceof Error ? error.message : String(error),
    } as ProverResponse);
  }
}

void initializeWasm();

self.onmessage = async (event: MessageEvent<ProverMessage>) => {
  const { type } = event.data;

  if (type === "params") {
    const { baseUrl } = event.data;

    const resolver = WasmResolver.new(baseUrl);
    const pp = MidnightWasmParamsProvider.new(baseUrl);

    prover = WasmProver.new(resolver, pp);

    self.postMessage({ type: "params-ready" });
  } else if (type === "prove") {
    const { serializedTx } = event.data;
    await runProver(serializedTx);
  }
};
