# Burn Tensor

> [Burn](https://github.com/burn-rs/burn) Tensor Library 

[![Current Crates.io Version](https://img.shields.io/crates/v/burn-tensor.svg)](https://crates.io/crates/burn-tensor)
[![license](https://shields.io/badge/license-MIT%2FApache--2.0-blue)](https://github.com/burn-rs/burn-tensor/blob/master/README.md)

This library provides multiple tensor implementations hidden behind an easy to use API that supports reverse mode automatic differentiation.

## Features

* Flexible ✨
* CPU + GPU 🙏
* Multi-Threads 🚀
* Intuitive Usage 😌
* No Global State 🚫
* Multiple Backends 🦾
* Reverse Mode Autodiff 🔥

### Backends

For now, only two backends are implemented, but adding new ones should not be that hard.

* [X] Pytorch using [tch-rs](https://github.com/LaurentMazare/tch-rs)
* [X] 100% Rust backend using [ndarray](https://github.com/rust-ndarray/ndarray)
* [X] [WGPU](https://github.com/gfx-rs/wgpu) backend
* [ ] Tensorflow using [tensorflow-rust](https://github.com/tensorflow/rust)
* [ ] CuDNN using RustCUDA[tensorflow-rust](https://github.com/Rust-GPU/Rust-CUDA)
* [ ] ...

### Autodiff

Automatic differentiation is implemented as just another tensor backend without any global state.
It's possible since we keep track of the order in which each operation as been executed and the tape is only created when calculating the gradients.
To do so, each operation creates a new node which has a reference to its parent nodes.
Therefore, creating the tape only requires a simple and efficient graph traversal algorithm.

```rust
    let x = ADTensor::from_tensor(x_ndarray);
    let y = ADTensor::from_tensor(y_ndarray);

    let z = x.matmul(&y);

    let grads = z.backward();

    let x_grad = x.grad(&grads);
    let y_grad = y.grad(&grads);
```

## Cuda

To run with CUDA set `TORCH_CUDA_VERSION=cu113`.

## Notes

This crate can be used alone without the entire burn stack and with only selected backends for smaller binaries.


## Feature Flags

This crate can be used without the standard library (`#![no_std]`) with `alloc` by disabling
the default `std` feature.

* `std` - enables the standard library.
* `burn-tensor-testgen` - enables test macros for generating tensor tests. 

