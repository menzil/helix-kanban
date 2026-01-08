# Homebrew Tap for helix-kanban

This is the official Homebrew tap for [helix-kanban](https://github.com/menzil/helix-kanban), a terminal-based kanban board with Helix-style keybindings.

## Installation

### Add the tap

```bash
brew tap menzil/tap
```

### Install helix-kanban

```bash
brew install helix-kanban
```

Or with a specific version:

```bash
brew install menzil/tap/helix-kanban@0.2.20
```

## Usage

After installation, run:

```bash
hxk
```

## Uninstallation

```bash
brew uninstall helix-kanban
```

To remove the tap:

```bash
brew untap menzil/tap
```

## Available Versions

- **0.2.20** - Latest stable release with dynamic status columns and column width adjustment

## Formula Development

### Building from source

The formula builds helix-kanban from source using Rust's Cargo. Make sure you have Rust installed:

```bash
brew install rust
```

### Testing

Test the installation:

```bash
brew test helix-kanban
```

### Formula Verification

Verify the formula:

```bash
brew audit --formula helix-kanban
brew style helix-kanban
```

## License

This tap and the helix-kanban formula are released under the same license as the project: MIT OR Apache-2.0
