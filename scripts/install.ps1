# Installation Guide

## Quick Install

### Linux/macOS
```bash
curl -sSL https://raw.githubusercontent.com/nnanto/local_search/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/nnanto/local_search/main/install.ps1 | iex
```

## Manual Installation

### Pre-built Binaries

Download the appropriate binary for your platform from the [latest release](https://github.com/nnanto/local_search/releases/latest):

#### Linux (x86_64)
```bash
curl -L https://github.com/nnanto/local_search/releases/latest/download/local-search-linux-x86_64.tar.gz | tar xz
sudo mv local-search /usr/local/bin/
```

#### macOS (Intel)
```bash
curl -L https://github.com/nnanto/local_search/releases/latest/download/local-search-macos-x86_64.tar.gz | tar xz
sudo mv local-search /usr/local/bin/
```

#### macOS (Apple Silicon)
```bash
curl -L https://github.com/nnanto/local_search/releases/latest/download/local-search-macos-aarch64.tar.gz | tar xz
sudo mv local-search /usr/local/bin/
```

#### Windows
1. Download [local-search-windows-x86_64.zip](https://github.com/nnanto/local_search/releases/latest/download/local-search-windows-x86_64.zip)
2. Extract the ZIP file
3. Add the extracted directory to your PATH environment variable

### From Source

If you have Rust installed, you can build from source:

```bash
cargo install --git https://github.com/nnanto/local_search --features cli
```

Or clone and build:

```bash
git clone https://github.com/nnanto/local_search.git
cd local_search
cargo build --release --features cli
sudo cp target/release/local-search /usr/local/bin/
```

## Verify Installation

After installation, verify that the tool is working:

```bash
local-search --help
```

You should see the help output for the local-search CLI tool.

## Updating

To update to the latest version, simply re-run the installation command. The installer will replace the existing binary with the latest version.

## Uninstallation

### Linux/macOS
```bash
sudo rm /usr/local/bin/local-search
```

### Windows
Remove the installation directory and update your PATH environment variable to remove the local-search directory.

## Troubleshooting

### Permission Issues
If you get permission errors on Linux/macOS, make sure you're running the installation with appropriate permissions (using `sudo` when needed).

### Path Issues
If the `local-search` command is not found after installation, make sure the installation directory is in your PATH:

- **Linux/macOS**: `/usr/local/bin` should be in your PATH
- **Windows**: The installation directory should be added to your PATH environment variable

### Download Issues
If you're having trouble downloading the binary, you can:
1. Check your internet connection
2. Try downloading manually from the [releases page](https://github.com/nnanto/local_search/releases/latest)
3. Use a VPN if you're in a region with restricted access

### Antivirus False Positives
Some antivirus software may flag the binary as suspicious. This is a common issue with Rust binaries. You may need to add an exception for the local-search binary.