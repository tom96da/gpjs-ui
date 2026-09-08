// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { defineConfig, mergeConfig } from "vite";

import viteConfig from "./vite.config.mts";

export default mergeConfig(
  viteConfig,
  defineConfig({
    resolve: { conditions: ["source"] },
    ssr: { resolve: { conditions: ["source"] } },
  }),
);
