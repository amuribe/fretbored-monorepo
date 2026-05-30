use wasm_bindgen::prelude::*;

pub mod instrument;
pub mod theory;

// Convert rust DB to JavaScript
#[wasm_bindgen]
pub fn get_instrument_database() -> Result<JsValue, JsValue> {
    // Load the instrument database from engine
    let db = instrument::InstrumentDatabase::load();

    // Serialize to JS Object
    serde_wasm_bindgen::to_value(&db.instruments).map_err(|err| err.to_string().into())
}

// Lookup for fretboard clicks, returns Note name as String (e.g. "C#")
#[wasm_bindgen]
pub fn get_note_name(
    instrument_name: &str,
    tuning_name: &str,
    string_index: usize,
    fret: u8,
) -> Option<String> {
    let db = instrument::InstrumentDatabase::load();
    let tuning = db.get_tuning(instrument_name, tuning_name)?;

    let note = tuning.note_at_fret(string_index, fret)?;

    // Format Note enum into string for JavaScript
    Some(format!("{:?}", note))
}
