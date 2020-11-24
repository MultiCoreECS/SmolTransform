# SmolTransform

An experiment for benchmarking hierarchal transforms with the SmolECS crate.

## Building

Install the Rust compiler, install Cargo the Rust package manager, and run

```
cargo build --release
```

the resulting executable will be under `target/release/`.

## Options

```
-c, --object_count          Sets the number of objects to generate
-i, --update_iterations     Sets the number of transform update iterations to perform
-t, --transform_type        Sets the type of transform. 0=all, 1=rotation, 2=scale, 3=translation
```
