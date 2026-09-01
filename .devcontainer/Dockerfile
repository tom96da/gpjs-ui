# syntax=docker/dockerfile:1

# Copyright (c) 2026 tom96da
# SPDX-License-Identifier: MIT OR Apache-2.0

FROM mcr.microsoft.com/devcontainers/rust:2-1-trixie

# Cache mounts keep apt's downloads across rebuilds instead of the image layer.
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update && \
    apt-get install -y --no-install-recommends \
      # build toolchain
      gcc \
      g++ \
      make \
      cmake \
      clang \
      lld \
      llvm \
      pkg-config \
      # font shaping
      libfontconfig-dev \
      # windowing (wayland/x11)
      libwayland-dev \
      libx11-xcb-dev \
      libxkbcommon-x11-dev \
      # tls
      libssl-dev \
      # vulkan
      libvulkan1 \
      mesa-vulkan-drivers \
      # headless display
      xvfb
