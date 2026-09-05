// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { defineConfig } from "oxlint";

export default defineConfig({
  plugins: ["eslint", "typescript", "unicorn", "oxc", "import", "node", "jsdoc", "vitest", "vue"],
  ignorePatterns: ["third_party/**"],
  rules: {
    "vue/prefer-import-from-vue": "off",
  },
});
