# Ascot Gateway

A gateway which performs the following tasks:

- Detecting devices in a trusted network
- Showing a panel to interact with devices and change their states
- Allowing to run commands on devices

# Building for ARM

Install `cross` tool

```console
cargo install cross
```

Build `ascot-gateway` for ARM

```console
cross build --release --target=arm-unknown-linux-gnueabihf
```
