# Fretbored

An interactive, data-driven fretboard visualizer for multiple string instruments, and a music theory engine.

The project is split bifurcated into a high-performance Rust core which handles the music theory logic, and a React frontend for interactive visualization, connected via WebAssembly (WASM).

## Architecture & Tech Stack

- **Core Engine:** Rust
- **Frontend:** React, TypeScript, Vite, Tailwind CSS v4
- **Bridge (Planned):** WebAssembly (`wasm-bindgen`)

## Current Progress (Work Completed)

### Rust Core Engine (`core_engine`)

- **Data-Driven Instruments:** Structured `Instrument` and `Tuning` models that load dynamically from an embedded `instruments.json` database.
- **Scientific Pitch Notation (SPN) Parser:** Implemented parsing (e.g., `C#4`, `Db3`) into MIDI numbers using a type-safe `MidiNote` wrapper.
- **Theory Foundations:** Basic implementations for `Note` and `Chord` identification.
- **Testing & Linting:** Basic unit testing for bounds checking, parsing boundaries, and SPN conversions. Linting enforced via `cargo clippy`.

### Frontend (`frontend`)

- **Scaffolding:** Initialized Vite + React + TypeScript environment.
- **Styling:** Configured Tailwind CSS v4.
- **Cleanup:** Purged default boilerplate and assets to establish a clean slate.

## Roadmap & Task List

### Phase 1: The WASM Bridge

- [x] Add `wasm-bindgen` to `Cargo.toml`.
- [x] Write wrapper functions in Rust to expose `InstrumentDatabase::load()` and `note_at_fret()`.
- [x] Compile the engine to a node module using `wasm-pack`.
- [x] Import and initialize the WASM module in the React frontend.

### Phase 2: Core UI Components

- [x] **State Management:** Create a custom React hook (e.g., `useInstrument`) to manage active tunings and interface with the WASM backend.
- [x] **Fretboard Grid:** Build a `<Fretboard />` component mapping over string arrays to render a responsive 0-24 fret grid.
- [x] **Data Binding:** Connect the UI grid to the Rust engine to display correct note names based on fret and string coordinates.

### Phase 3: Engine Expansion (Theory)

- [ ] **Intervals:** Create an `interval.rs` module to define distances between notes (e.g., Minor 3rd).
- [ ] **Scales:** Create a `scale.rs` module to generate scales (Major, Minor, Pentatonic) using root notes and interval formulas.
- [ ] **Scale Mapping:** Write a function to calculate all `(string, fret)` coordinates for a given scale across an active instrument tuning.

### Phase 4: Audio Synthesis

- [ ] Implement the Web Audio API in the frontend.
- [ ] Map outputted `MidiNote` values to exact frequencies.
- [ ] Generate basic sine/triangle wave tones on user interaction.

### Phase 5: Database Expansion

Once the core visualization and theory engine are stable, expand the static data catalog to support a wider range of musical applications.

- [ ] **Instruments:** Add support for extended-range guitars (7-string, 8-string), ukuleles, banjos, and mandolins.
- [ ] **Tunings:** Populate `instruments.json` with alternate and open tunings (e.g., DADGAD, Open G, Drop C).
- [ ] **Chord Library:** Expand the chord recognition engine to include 7ths (Maj7, Min7, Dom7), extended chords (9ths, 11ths, 13ths),altered chords, etc.
- [ ] **Scale Library:** Add the 7 modes of the Major scale, Harmonic Minor, Melodic Minor, and common exotic scales.

## Getting Started

### Prerequisites

- Rust (Cargo)
- Node.js & npm

### Running the Backend Tests

```bash
cd core_engine
cargo test
```
