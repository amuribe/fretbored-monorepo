# Fretbored

An advanced, high-performance music theory engine and fretboard visualizer.

The project is driven by a high-performance, zero-allocation Rust core which handles the complex music theory and instrument logic. Interaction is currently handled via a stylized command-line interface (CLI), with WebAssembly (WASM) and React frontend integration paused for future development.

## Architecture & Tech Stack

- **Core Engine:** Rust (Zero-allocation domain models, static data)
- **Interface:** Interactive stylized CLI (Rust)
- **Frontend (Paused):** React, TypeScript, Vite, CSS Modules, WebAssembly

## Current Progress (Work Completed)

### Rust Core Engine (`core_engine`)

- **Zero-Allocation Architecture:** Replaced runtime file I/O and JSON parsing with compile-time static evaluation. Domain constants live in the binary's `.rodata` segment.
- **Instrument Registry:** Implemented a static `TuningRegistry` for $O(1)$ lookups of standard and alternate tunings across Guitar, Bass, and Ukulele.
- **Advanced Theory Engine:** - **Notes & Enharmonics:** Complete Scientific Pitch Notation (SPN) parsing to MIDI, with full support for note spellings and enharmonic equivalents (sharps, flats).
  - **Intervals & Scales:** Modular interval math driving a generator for Major, Minor, and Pentatonic scales.
  - **Composable Chords:** Highly scalable chord architecture separating base triads (`ChordQuality`) from modular `Extension`s (6ths, 7ths, 9ths, 11ths, 13ths, and alterations).
  - **Mapping Engine:** Implemented an $O(1)$ bitmask search algorithm to map theoretical note clusters onto physical coordinate matrices.

---

## Roadmap & Task List

### Phase 1: Theory Engine (Complete)

- [x] Establish base `Note`, `MidiNote`, and `Interval` math using modulo 12 logic.
- [x] Implement enharmonic spelling logic (`NoteSpelling`).
- [x] Build scale generator (`ScaleType`).
- [x] Overhaul chord engine to a composable `ChordQuality` + `Extension` struct.

### Phase 2: Instrument Data (Complete)

- [x] Extract `Tuning` and `FretBoardConfig` into zero-allocation domain types.
- [x] Hardcode physical instrument structures into a `TuningRegistry`.

### Phase 3: The Mapping Algorithm (Complete)

- [x] **Coordinate Matrix:** Write the core search algorithm (`find_notes_on_fretboard`) that takes a generated `Chord` or Scale `Vec<Note>` and scans a `FretBoardConfig` to return all physical `(string_index, fret_number)` coordinates.

### Phase 4: Voicing Engine (Current Phase)

- [ ] Build a filter to analyze raw fretboard coordinates and output ergonomic, playable chord voicings based on physical hand-stretch constraints.

### Phase 5: Stylized CLI

- [ ] Scaffold interactive command-line interface.
- [ ] Implement query menus for tuning selection and chord/scale generation.
- [ ] Build terminal renderer for the 2D fretboard coordinate matrix.

### Phase 6: Deep Theory & DSP Expansion

- [ ] **Advanced Harmonic Structures:** Implement Slash Chords (independent bass note evaluation) and Inversions (intervallic rotation and recalculation).
- [ ] **Modes & Global Scales:** Expand the `ScaleType` generator to support non-diatonic and synthetic scale degrees (e.g., Harmonic Minor modes, Hirajoshi, Double Harmonic).
- [ ] **Frequency Matrix (DSP Prep):** Map absolute MIDI note values to exact mathematical frequencies, supporting variable Equal Temperament reference pitches (e.g., A=440 Hz vs A=432 Hz).
- [ ] **Macro-Theory & Functional Harmony:** Build a Roman Numeral analysis parser to define chord progressions and functional relationships within a designated key.
- [ ] **Voice Leading Algorithm:** Implement pathfinding to calculate the most efficient physical and theoretical movement between consecutive chord voicings in a progression.

### Phase 7: WebAssembly & Frontend UI (Paused)

- [ ] Re-link WASM boundary to expose the completed engine APIs (Voicing, Progressions, Frequencies).
- [ ] Finalize React coordinate-binding state management.
- [ ] Implement Web Audio API synthesis using the Phase 6 frequency matrix.

## Getting Started

### Prerequisites

- Rust (Cargo)

### Run the Engine (CLI)

```bash
cd core_engine
cargo run
```
