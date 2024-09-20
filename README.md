# ICP Rust NFT Boilerplate Backend

This project is a backend boilerplate for creating and managing NFT certificates on the Internet Computer (ICP) network using Rust.

## Features

- **Create NFT**: Create a new NFT certificate with owner information and metadata.
- **Get NFT**: Retrieve NFT certificate information by ID.
- **Delete NFT**: Delete an NFT certificate by ID.

## Project Structure

- `src/lib.rs`: The main file containing the logic for creating, retrieving, and deleting NFT certificates.

## Installation

1. **Clone the repository**:

   ```sh
   git clone https://github.com/username/icp-rust-boilerplate-backend.git
   cd icp-rust-boilerplate-backend
   ```

2. **Install dependencies**:
   Ensure you have [Rust](https://www.rust-lang.org/tools/install) and [DFX](https://smartcontracts.org/docs/developers-guide/install-upgrade-remove.html) installed.

   ```sh
   dfx start --background
   dfx deploy
   ```
