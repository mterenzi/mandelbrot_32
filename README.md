# Mandelbrot 32

A high-performance, hardware-accelerated Mandelbrot set renderer written in Rust using [wgpu](https://wgpu.rs/).

## Overview

This project renders the Mandelbrot set in real-time using the GPU. It leverages `wgpu` for cross-platform graphics API support (Vulkan, Metal, DX12, WebGPU) and `winit` for window management. The rendering logic is implemented via a WGSL fragment shader, allowing for smooth zooming and panning.

## Features

*   **GPU Acceleration**: Utilizes the GPU for parallel fractal calculation, ensuring high frame rates.
*   **32-bit Precision**: Renders using 32-bit floating point precision.
*   **Real-time Rendering**: Supports dynamic updates to zoom and position.
*   **Cross-Platform**: Runs on Linux, Windows, and macOS thanks to the portable `wgpu` backend.

## Prerequisites

*   Rust (Latest stable toolchain).
*   A graphics card compatible with Vulkan, Metal, DirectX 12, or WebGPU.

## Getting Started

1.  **Clone the repository:**

    ```bash
    git clone <repository_url>
    cd mandelbrot
    ```

2.  **Run the project:**

    It is highly recommended to run in release mode for optimal performance.

    ```bash
    cargo run --release
    ```

## Project Structure

*   **`src/wgpu.rs`**: Handles the WGPU context setup, surface configuration, render pipeline creation, and the main render pass.
*   **`src/primitives.rs`**: Defines the data structures (`Vertex`, `Uniforms`) shared between the CPU and the GPU.
*   **`src/shaders/mandelbrot_32.wgsl`**: The shader source code responsible for calculating the Mandelbrot iterations per pixel.

## Dependencies

*   `wgpu`: Graphics API abstraction.
*   `winit`: Window creation and event handling.
*   `bytemuck`: Utilities for casting data to bytes for GPU buffers.
*   `pollster`: Simple async executor for WGPU initialization.
