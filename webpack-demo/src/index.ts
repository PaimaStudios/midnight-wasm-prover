import testVectors from "./test-vectors.js";
import type { ProverMessage, ProverResponse } from "./worker-types.js";

const worker = new Worker(new URL("./prover-worker.ts", import.meta.url));

function uint8ArrayToHex(uint8Array: Uint8Array) {
  return Array.from(uint8Array, function(byte) {
    return ("0" + (byte & 0xff).toString(16)).slice(-2);
  }).join("");
}

async function printElapsedTime(
  startTime: number,
  abortController: AbortController,
  intervalMs = 1000
): Promise<void> {
  const timeLabel = document.getElementById('timeLabel') as HTMLElement;

  while (!abortController.signal.aborted) {
    const elapsedMs = Date.now() - startTime;
    const seconds = Math.floor(elapsedMs / 1000);
    const minutes = Math.floor(seconds / 60);
    const timeText = `Elapsed time: ${minutes}m ${seconds % 60}s`;

    if (timeLabel) {
      timeLabel.textContent = timeText;
    }

    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
}

async function initializeWorker() {
  const baseUrl = new URL(window.location.href).toString();

  let readyResolve: (value: void) => void;
  let paramsResolve: (value: void) => void;

  const wasmReady = new Promise<void>((resolve, _reject) => {
    readyResolve = resolve;
  });
  const paramsReady = new Promise<void>((resolve, _reject) => {
    paramsResolve = resolve;
  });

  worker.onmessage = (event: MessageEvent<ProverResponse>) => {
    const { type, message } = event.data;

    switch (type) {
      case "wasm-ready":
        readyResolve();
        break;
      case "params-ready":
        paramsResolve();
        break;
      case "error":
        console.error(message);
        break;
    }
  };

  await wasmReady;

  worker.postMessage({
    type: "params",
    baseUrl,
  } as ProverMessage);

  await paramsReady;
}

async function runProof() {
  const button = document.getElementById('proveButton') as HTMLButtonElement;
  const timeLabel = document.getElementById('timeLabel') as HTMLElement;
  const statusLabel = document.getElementById('statusLabel') as HTMLElement;
  const resultLabel = document.getElementById('resultLabel') as HTMLElement;

  button.disabled = true;
  button.textContent = 'Proving...';
  timeLabel.textContent = 'Elapsed time: 0m 0s';
  statusLabel.textContent = 'Proving in progress...';
  statusLabel.style.color = 'blue';
  resultLabel.textContent = '';

  const startTime = Date.now();
  const abortController = new AbortController();
  const timeTrackerPromise = printElapsedTime(startTime, abortController);

  let result = new Promise((resolve, reject) => {
    worker.onmessage = (event: MessageEvent<ProverResponse>) => {
      const { type, data, message } = event.data;

      switch (type) {
        case "success":
          abortController.abort();
          button.disabled = false;
          button.textContent = 'Start Proof';
          const duration = event.data.durationMs;
          statusLabel.textContent = `Proof completed successfully! (${duration}ms)`;
          statusLabel.style.color = 'green';
          resultLabel.textContent = `${uint8ArrayToHex(data)}`;
          resolve(data);
          break;
        case "error":
          abortController.abort();
          button.disabled = false;
          button.textContent = 'Start Proof';
          statusLabel.textContent = `Proof failed: ${message}`;
          statusLabel.style.color = 'red';
          reject(new Error(message));
          break;
      }
    };

    worker.onerror = (error) => {
      abortController.abort();
      button.disabled = false;
      button.textContent = 'Start Proof';
      statusLabel.textContent = `Worker error: ${error.message}`;
      statusLabel.style.color = 'red';
      reject(error);
    };

    worker.onmessageerror = () => {
      abortController.abort();
      button.disabled = false;
      button.textContent = 'Start Proof';
      statusLabel.textContent = 'Worker message error';
      statusLabel.style.color = 'red';
      reject(new Error('Worker message error'));
    };
  });

  let unproven = testVectors.unprovenTransactionGuaranteed();

  worker.postMessage({
    type: "prove",
    serializedTx: unproven.serialize(),
  } as ProverMessage);

  await Promise.race([result, timeTrackerPromise]);
  await result;
}

function downloadBytes(filename: string, bytes: Uint8Array) {
  const copied = new Uint8Array(bytes);
  const blob = new Blob([copied.buffer], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function dumpUnprovenTx() {
  const unproven = testVectors.unprovenTransactionGuaranteed();
  const serialized = unproven.serialize();
  downloadBytes('unproven-transaction.bin', serialized);
}

async function run() {
  const container = document.createElement('div');
  container.style.padding = '20px';
  container.style.fontFamily = 'Arial, sans-serif';

  const actions = document.createElement('div');
  actions.style.display = 'flex';
  actions.style.gap = '12px';
  actions.style.alignItems = 'center';
  actions.style.margin = '10px 0';
  actions.style.flexWrap = 'wrap';

  const button = document.createElement('button');
  button.id = 'proveButton';
  button.textContent = 'Initializing...';
  button.disabled = true;
  button.style.padding = '10px 20px';
  button.style.fontSize = '16px';

  const dumpButton = document.createElement('button');
  dumpButton.id = 'dumpButton';
  dumpButton.textContent = 'Dump Unproven Tx';
  dumpButton.style.padding = '10px 20px';
  dumpButton.style.fontSize = '16px';
  dumpButton.onclick = dumpUnprovenTx;

  const timeLabel = document.createElement('div');
  timeLabel.id = 'timeLabel';
  timeLabel.textContent = 'Elapsed time: 0m 0s';
  timeLabel.style.fontSize = '14px';
  timeLabel.style.margin = '10px 0';
  timeLabel.style.fontWeight = 'bold';

  const statusLabel = document.createElement('div');
  statusLabel.id = 'statusLabel';
  statusLabel.textContent = 'Ready to start proof';
  statusLabel.style.fontSize = '14px';
  statusLabel.style.margin = '10px 0';
  statusLabel.style.fontWeight = 'bold';
  statusLabel.style.color = 'gray';

  const resultLabel = document.createElement('div');
  resultLabel.id = 'resultLabel';
  resultLabel.textContent = '';
  resultLabel.style.fontSize = '12px';
  resultLabel.style.margin = '10px 0';
  resultLabel.style.fontFamily = 'monospace';
  resultLabel.style.wordBreak = 'break-all';
  resultLabel.style.backgroundColor = '#f5f5f5';
  resultLabel.style.padding = '10px';
  resultLabel.style.border = '1px solid #ddd';
  resultLabel.style.borderRadius = '4px';
  resultLabel.style.maxHeight = '200px';
  resultLabel.style.overflowY = 'auto';

  actions.appendChild(button);
  actions.appendChild(dumpButton);
  container.appendChild(actions);
  container.appendChild(timeLabel);
  container.appendChild(statusLabel);
  container.appendChild(resultLabel);
  document.body.appendChild(container);

  await initializeWorker();

  button.disabled = false;
  button.textContent = 'Start Proof';
  button.onclick = runProof;
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", run);
} else {
  run();
}
