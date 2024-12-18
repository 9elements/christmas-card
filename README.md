
# Building and Flashing the Project on a Raspberry Pi Pico

This guide will walk you through the steps to build and flash the project onto a Raspberry Pi Pico using Rust and Cargo.

## Prerequisites

Before you begin, ensure you have the following installed on your system:

1. **Rust and Cargo**: Rust is a systems programming language, and Cargo is its package manager and build system. You can install Rust and Cargo by following the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

2. **Additional Tools**: You will need some additional tools to build and flash the project:
   - `elf2uf2-rs`: A tool to convert ELF files to UF2 format for flashing onto the Raspberry Pi Pico.
   - Optionally `probe-rs`: A tool for on-chip debugging, flashing, and more, specifically designed for embedded systems.

## Setting Up the Environment

1. **Install Rust and Cargo**:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Install Additional Tools**:

   ```bash
   cargo install elf2uf2-rs
   cargo install probe-rs
   ```

## Building the Project

To build the project for the Raspberry Pi Pico, use the following command:

```bash
cargo run --release
```

After building the project, the resulting binary will be output in the `target/thumbv6m-none-eabi/release` directory. The binary will be in the UF2 format, which is suitable for flashing onto the Raspberry Pi Pico.

## Flashing the UF2 Binary

To flash the UF2 binary onto the Raspberry Pi Pico, follow these steps:

1. **Prepare the Raspberry Pi Pico**:
   - Hold down the BOOTSEL button on the Raspberry Pi Pico.
   - While holding the button, connect the Pico to your computer using a USB cable. This will bring the Pico into mass storage mode, and it should appear as a removable drive on your computer.

2. **Copy the UF2 File**:
   - Navigate to the `target/thumbv6m-none-eabi/release` directory.
   - Drag and drop the generated `.uf2` file onto the Raspberry Pi Pico's removable drive.

3. **Complete the Flashing Process**:
   - Once the file is copied, the Raspberry Pi Pico will automatically reboot and start running the new firmware.
