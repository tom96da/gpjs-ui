// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import path from "node:path";

import dts from "unplugin-dts/vite";
import { defineConfig } from "vitest/config";

export default defineConfig(({ mode }) => ({
  build: {
    lib: {
      entry: path.resolve(import.meta.dirname, "src/index.mts"),
      formats: ["es"],
      fileName: "index",
    },
    rollupOptions: {
      external: ["@vue/runtime-core"],
    },
  },
  plugins: [dts({ include: ["src"], exclude: ["src/**/*.test.mts"] })],
  // Only for `vitest` (mode "test"): resolve gpjs-ui from source.
  resolve: mode === "test" ? { conditions: ["source"] } : undefined,
  ssr: mode === "test" ? { resolve: { conditions: ["source"] } } : undefined,
  test: {
    passWithNoTests: true,
  },
}));
