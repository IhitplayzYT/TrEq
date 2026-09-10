# TrEq - Terminal Equalizer for PipeWire

A highly customizable, hierarchically cascaded configuration TUI application for creating and managing EQ (Equalizer) nodes in PipeWire audio graphs.

## Why TrEq?

TrEq provides a powerful terminal-based interface for designing complex audio equalizer chains with hierarchical node structures. Unlike traditional EQ tools that offer linear band configurations, TrEq allows you to:

- **Cascade EQ nodes hierarchically** - Build complex filter chains by nesting EQ nodes within each other
- **Design with precision** - Support for multiple filter types (Peaking, LowShelf, HighShelf, LowPass, HighPass, BandPass, Notch)
- **Point and range frequencies** - Define bands at specific frequencies or across frequency ranges
- **Persistent configurations** - Save and load EQ profiles as JSON files
- **Real-time biquad processing** - Built-in biquad filter implementation with coefficient calculation
- **Terminal-based workflow** - Efficient, keyboard-driven interface using ratatui TUI framework

TrEq is ideal for audio engineers, developers working with PipeWire, and anyone who needs fine-grained control over audio equalization in a Linux environment.

## Features

- **Hierarchical EQ Node Structure**: Create parent-child relationships between EQ nodes for complex signal processing chains
- **Multiple Filter Types**: 
  - Peaking
  - LowShelf
  - HighShelf
  - LowPass
  - HighPass
  - BandPass
  - Notch
- **Flexible Frequency Definition**: Support for both point frequencies and frequency ranges
- **Biquad Filter Implementation**: Accurate digital filter coefficients calculation and processing
- **Configuration Persistence**: Save/load EQ profiles as JSON files
- **CLI Interface**: Command-line argument parsing for profile selection and debug mode
- **Modular Architecture**: Clean separation of concerns with dedicated modules for models, DAO, filters, rendering, and input handling

## Dependencies

### Runtime Dependencies
- **PipeWire**: Audio server (required for the target use case)
- **Linux OS**: The application is designed for Linux environments

### Build Dependencies (Cargo.toml)
- `uuid` (v1.0.0) - UUID generation for unique identifiers
- `serde` (v1) - Serialization/deserialization framework
- `serde_json` (v1) - JSON serialization support
- `ratatui` (v0.29) - Terminal UI framework
- `ratatui-sci-fi` (v0.2.1) - Sci-fi themed TUI widgets
- `anyhow` (v1) - Error handling
- `crossterm` (v0.29) - Cross-platform terminal manipulation

## Installation

### Prerequisites
Ensure you have Rust installed on your system:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build from Source
```bash
# Clone the repository
git clone <repository-url>
cd TrEq

# Build the project
cargo build --release

# The binary will be available at target/release/TrEq
```

### Development Build
```bash
cargo build
```

## Usage

### Command-Line Interface

```bash
# Run with default settings
./target/release/TrEq

# Run in debug mode (prints CLI arguments)
./target/release/TrEq -d
./target/release/TrEq --debug

# Load a specific EQ profile
./target/release/TrEq --profile=my_profile.json
./target/release/TrEq -p=my_profile.json

# Specify custom profiles directory
./target/release/TrEq --db_path=/path/to/profiles
./target/release/TrEq -db=/path/to/profiles

# Show help
./target/release/TrEq -h
./target/release/TrEq --help
```

### CLI Arguments

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-d`, `--debug` | Enable debug mode (prints parsed arguments) |
| `-h`, `--help` | Display help information |
| `-p=`, `--profile=` | Load a specific EQ profile from file |
| `-db=`, `--db_path=` | Set custom directory for profile storage |

### Default Behavior

- **Profiles Directory**: `.Eqfiles/` in the current working directory
- **Profile Format**: JSON files containing serialized `EqProfile` structures
- **Auto-creation**: The profiles directory is created automatically if it doesn't exist

## Project Structure

```
TrEq/
├── src/
│   ├── main.rs          # Entry point and CLI argument parsing
│   ├── models.rs        # Core data structures (EqProfile, EqNode, EqBand, filters)
│   ├── Dao.rs           # Data access object for profile persistence
│   ├── filters.rs       # Biquad filter implementation and processing
│   ├── helper.rs        # CLI helper and argument parsing
│   ├── input.rs         # Input handling (TUI keyboard events)
│   ├── render.rs        # TUI rendering logic
│   └── processing.rs    # Audio processing pipeline
├── Cargo.toml           # Project dependencies and metadata
├── LICENSE              # GPL-3.0 license
└── README.md            # This file
```

## Core Concepts

### EqProfile
The top-level container for an equalizer configuration. Contains:
- Unique ID
- Profile name
- Preamp gain (optional)
- Root EQ node (hierarchical tree structure)
- Map of all nodes for quick lookup

### EqNode
A single EQ node in the hierarchical structure. Each node can:
- Contain multiple EQ bands
- Have child nodes (creating cascaded processing chains)
- Be enabled/disabled independently
- Store metadata (name, ID)

### EqBand
An individual frequency band with:
- Filter type (Peaking, LowShelf, etc.)
- Frequency (point or range)
- Gain value
- Q factor (bandwidth)
- Sample rate
- Enable/disable state

### Biquad Filters
Digital biquad filter implementation with:
- Coefficient calculation for all filter types
- Direct Form II transposed structure
- Stateful processing (maintains past samples)
- Layer processing for cascaded filters

## Examples

### Creating a Simple EQ Profile

```rust
use crate::models::models::{EqProfile, EqNode, EqBand, EqFilter, Freq};

// Create a new profile
let mut profile = EqProfile::new("My EQ Profile", Some(0.0));

// Create a root node
let root_node = EqNode::new(Some("Root EQ".to_string()));

// Add bands to the node
root_node.add_band(
    Freq::Point(1000), 
    Some(EqFilter::Peaking), 
    Some(3.0), 
    48000, 
    1.0
);

root_node.add_band(
    Freq::Point(100), 
    Some(EqFilter::LowShelf), 
    Some(2.0), 
    48000, 
    0.7
);

// Add node to profile
profile.add_node(None, root_node);

// Finalize the profile
profile.finalize();
```

### Hierarchical Node Structure

```rust
// Create parent node
let parent_node = EqNode::new(Some("Parent EQ".to_string()));
parent_node.add_band(Freq::Point(1000), Some(EqFilter::Peaking), Some(2.0), 48000, 1.0);

// Create child node
let child_node = EqNode::new(Some("Child EQ".to_string()));
child_node.add_band(Freq::Point(5000), Some(EqFilter::HighShelf), Some(-1.5), 48000, 0.5);

// Add child to parent
let child_id = child_node.id;
profile.add_node(Some(parent_node.id), child_node);

// The child node will be processed after the parent node
```

### Using the DAO for Persistence

```rust
use crate::Dao::dao::Dao;

// Initialize DAO with directory
let dao = Dao::new(".Eqfiles/".to_string());

// Save a profile
dao.create_config("my_profile.json", &profile).unwrap();

// Load all available configurations
let mut dao = Dao::new(".Eqfiles/".to_string());
dao.load_all_confs();

// Load a specific profile
if let Some(loaded_profile) = dao.load_conf("my_profile.json").unwrap() {
    println!("Loaded profile: {}", loaded_profile.get_name());
}

// Delete a profile
dao.delete_conf("old_profile.json").unwrap();
```

### Biquad Filter Processing

```rust
use crate::filters::filters::{Biquad, EqFilter, Freq};

// Create a biquad filter
let mut biquad = Biquad::new(
    EqFilter::Peaking,
    Freq::Point(1000),
    48000,
    1.0,
    Some(3.0)
);

// Process audio samples
let input_sample = 0.5f32;
let output_sample = biquad.biquad_pass_mut(input_sample);

// Reset filter state
biquad.reset();

// Layer multiple biquads
let mut biquads = vec![
    Biquad::new(EqFilter::LowShelf, Freq::Point(100), 48000, 0.7, Some(2.0)),
    Biquad::new(EqFilter::Peaking, Freq::Point(1000), 48000, 1.0, Some(3.0)),
    Biquad::new(EqFilter::HighShelf, Freq::Point(5000), 48000, 0.5, Some(-1.5)),
];

let processed = Biquad::layer_biquad_pass(input_sample, &mut biquads);
```

### Filter Types Reference

| Filter Type | Description | Typical Use Case |
|-------------|-------------|------------------|
| **Peaking** | Boost or cut a frequency range | EQ adjustments, tone shaping |
| **LowShelf** | Boost or cut all frequencies below a point | Bass enhancement, warmth |
| **HighShelf** | Boost or cut all frequencies above a point | Airiness, brilliance |
| **LowPass** | Attenuate frequencies above a cutoff | Anti-aliasing, removing highs |
| **HighPass** | Attenuate frequencies below a cutoff | Removing rumble, DC offset |
| **BandPass** | Allow only a specific frequency range | Isolating frequency bands |
| **Notch** | Attenuate a very narrow frequency range | Removing hum, feedback |

## Configuration File Format

EQ profiles are stored as JSON files with the following structure:

```json
{
  "id": "uuid-v4",
  "name": "My EQ Profile",
  "preamp": 0.0,
  "root": {
    "id": "uuid-v4",
    "name": "Root EQ",
    "enabled": true,
    "bands": [
      {
        "id": "uuid-v4",
        "enabled": true,
        "filter": "Peaking",
        "sample_rate": 48000,
        "q": 1.0,
        "freq": {"Point": 1000},
        "gain": 3.0
      }
    ],
    "children": [],
    "band_map": {}
  },
  "nodes": {}
}
```

## Development

### Running Tests
```bash
cargo test
```

### Checking Code
```bash
cargo check
```

### Formatting Code
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## License

This project is licensed under GPL-3.0-only. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust best practices
- All tests pass
- New features include documentation
- Commits are clear and descriptive

## Roadmap

- [ ] Complete TUI implementation with interactive node/band editing
- [ ] Add real-time audio processing integration with PipeWire
- [ ] Implement preset library with common EQ curves
- [ ] Add frequency response visualization
- [ ] Support for importing/exporting EQ profiles from other tools
- [ ] Add configuration validation and error handling
- [ ] Implement undo/redo functionality
- [ ] Add keyboard shortcuts reference in TUI

## Troubleshooting

### Build Errors
If you encounter build errors, ensure:
- Rust toolchain is up to date: `rustup update`
- All dependencies are available: `cargo fetch`

### Runtime Issues
- Ensure PipeWire is running: `systemctl --user status pipewire`
- Check that the profiles directory exists and is writable
- Use debug mode (`-d`) to inspect CLI argument parsing

## Acknowledgments

Built with:
- [Ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [Ratatui Sci-Fi](https://github.com/ratatui-org/ratatui-sci-fi) - Sci-fi themed widgets
- [PipeWire](https://pipewire.org/) - Audio server
