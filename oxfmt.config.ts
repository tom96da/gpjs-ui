// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { defineConfig } from "oxfmt";

export default defineConfig({
  ignorePatterns: ["**.md", "third_party/**"],
  sortImports: {
    order: "asc",
    ignoreCase: true,
    newlinesBetween: true,
    customGroups: [
      {
        groupName: "first-party",
        elementNamePattern: ["gpjs-ui*"],
        modifiers: ["value"],
      },
      {
        groupName: "type-first-party",
        elementNamePattern: ["gpjs-ui*"],
        modifiers: ["type"],
      },
      {
        groupName: "scoped-first-party",
        elementNamePattern: ["@gpjs-ui/*"],
        modifiers: ["value"],
      },
      {
        groupName: "type-scoped-first-party",
        elementNamePattern: ["@gpjs-ui/*"],
        modifiers: ["type"],
      },
    ],
    groups: [
      "builtin",
      "external",
      "first-party",
      { newlinesBetween: false },
      "scoped-first-party",
      ["internal", "subpath"],
      ["parent", "sibling", "index"],

      "type-builtin",
      "type-external",
      "type-first-party",
      { newlinesBetween: false },
      "type-scoped-first-party",
      ["type-internal", "type-subpath"],
      { newlinesBetween: false },
      ["type-parent", "type-sibling", "type-index"],

      "style",
      "unknown",
    ],
  },
});
