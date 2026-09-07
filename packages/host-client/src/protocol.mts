// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

// The wire shapes this package reads off the host's stdout. Internal only —
// `HostClient` is what a caller talks to; nothing here is re-exported from
// `index.mts`.

export const JSONRPC = "2.0";

export interface RpcResult {
  jsonrpc: typeof JSONRPC;
  id: number;
  result: unknown;
}

export interface RpcFailure {
  jsonrpc: typeof JSONRPC;
  id: number;
  error: { code: number; message: string; data?: { stack: string | null } };
}

export interface RpcNotification {
  jsonrpc: typeof JSONRPC;
  method: string;
  params: unknown;
}

export function isRpcMessage(value: unknown): value is RpcResult | RpcFailure | RpcNotification {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as { jsonrpc?: unknown }).jsonrpc === JSONRPC
  );
}
