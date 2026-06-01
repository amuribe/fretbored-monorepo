# Fretbored

An interactive, data-driven fretboard visualizer and advanced music theory engine.

The project is bifurcated into a high-performance, zero-allocation Rust core which handles the complex music theory and instrument logic, and a React frontend for interactive visualization, connected seamlessly via WebAssembly (WASM).

## Architecture & Tech Stack

- **Core Engine:** Rust (Zero-allocation domain models, static data)
- **Frontend:** React, TypeScript, Vite, CSS Modules
- **Bridge:** WebAssembly (`wasm-bindgen`, `serde-wasm-bindgen`)

## Current Progress (Work Completed)

### Rust Core Engine (`core_engine`)

- **Zero-Allocation Architecture:** Replaced runtime file I/O and JSON parsing with compile-time static evaluation. Domain constants live in the binary's `.rodata` segment.
- **Instrument Registry:** Implemented a static `TuningRegistry` for $O(1)$ lookups of standard and alternate tunings across Guitar, Bass, and Ukulele.
- **Advanced Theory Engine:** - **Notes & Enharmonics:** Complete Scientific Pitch Notation (SPN) parsing to MIDI, with full support for note spellings and enharmonic equivalents (sharps, flats).
  - **Intervals & Scales:** Modular interval math driving a generator for Major, Minor, and Pentatonic scales.
  - **Composable Chords:** Highly scalable chord architecture separating base triads (`ChordQuality`) from modular `Extension`s (6ths, 7ths, 9ths, 11ths, 13ths, and alterations).
- **WASM Bridge:** Engine logic successfully exported to WebAssembly, exposing `get_note_at_fret` and `list_tunings` APIs to the frontend.

### Frontend (`frontend`)

- **Scaffolding:** Initialized Vite + React + TypeScript environment.
- **Styling:** Configured CSS Modules for scoped, collision-free component styling.
- _Note: The frontend UI is currently decoupled pending rewiring to the new WASM architecture._

---

## Roadmap & Task List

### Phase 1: Theory Engine (Complete)

- [x] Establish base `Note`, `MidiNote`, and `Interval` math using modulo 12 logic.
- [x] Implement enharmonic spelling logic (`NoteSpelling`).
- [x] Build scale generator (`ScaleType`).
- [x] Overhaul chord engine to a composable `ChordQuality` + `Extension` struct.

### Phase 2: Instrument Data & WASM Bridge (Complete)

- [x] Extract `Tuning` and `FretBoardConfig` into zero-allocation domain types.
- [x] Hardcode physical instrument structures into a `TuningRegistry`.
- [x] Add `wasm-bindgen` and compile the engine to a node module.
- [x] Expose tuning lookups and fretboard note calculation to the frontend.

### Phase 3: The Mapping Algorithm (Current Phase)

- [ ] **Coordinate Matrix:** Write the core search algorithm (`find_notes_on_fretboard`) that takes a generated `Chord` or Scale `Vec<Note>` and scans a `FretBoardConfig` to return all physical `(string_index, fret_number)` coordinates.

### Phase 4: Frontend Integration & UI

- [ ] **State Management:** Write a custom React hook to interface with the new WASM API.
- [ ] **Fretboard Grid:** Build a responsive `<Fretboard />` component mapping the physical dimensions of the active tuning.
- [ ] **Data Binding:** Connect the UI grid to the Rust engine's mapping matrix to dynamically plot scales and chords.

### Phase 5: Voicing Engine (Advanced Theory)

- [ ] Build a filter to analyze raw fretboard coordinates and output ergonomic, playable chord voicings based on physical hand-stretch constraints.

### Phase 6: Audio Synthesis

- [ ] Implement the Web Audio API in the frontend.
- [ ] Map outputted `MidiNote` values to exact frequencies.
- [ ] Generate standard instrument tones on user interaction.

## Getting Started

### Prerequisites

- Rust (Cargo)
- `wasm-pack` (Install via `cargo install wasm-pack`)
- Node.js & npm

### 1. Build the Core Engine (WASM)

Compile the Rust core into a WebAssembly module targeting the web.

```bash
cd core_engine
cargo test
wasm-pack build --target web
```
