export type ProverMessage =
    | {
          type: "params";
          baseUrl: string;
      }
    | {
          type: "prove";
          serializedTx: Uint8Array;
      };

export interface ProverResponse {
    type: "success" | "error" | "log" | "wasm-ready" | "params-ready";
    data?: any;
    message?: string;
    durationMs?: number;
}
