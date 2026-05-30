import { useState, useEffect } from "react";
import InstrumentString from "./InstrumentString";
import InstrumentControls from "./InstrumentControls";
// Custom types
import type { Instrument } from "../../types/instrument";


interface FretBoardProps {
    // Database holding tuning and instruments
    db: Instrument[];
}

export default function Fretboard({ db }: FretBoardProps) {

    // Default instrument state, first instrument in database if exists else empty string
    const [activeInstrumentName, setActiveInstrumentName] = useState<string>(
        db.length > 0 ? db[0].name : ""
    );
    // Default tuning state (standard tuning for the instrument)
    const [activeTuningName, setActiveTuningName] = useState<string>("Standard");

    const availableInstruments = db.map((inst) => inst.name);

    // Get instrument object from the currently selected instrument in the dropdown (default first inst)
    const activeInstrumentObj = db.find((inst) => inst.name === activeInstrumentName) || db[0];
    // Get available tunings for instrument
    const availableTunings = activeInstrumentObj ? activeInstrumentObj.tunings.map((t) => t.name) : [];

    // If the tuning doesn't exist for the new instrument selected, fallback to its first available tuning.
    useEffect(() => {
        if (!availableTunings.includes(activeTuningName) && availableTunings.length > 0) {
            setActiveTuningName(availableTunings[0]);
        }
    }, [activeInstrumentName, availableTunings, activeTuningName]);

    // Calculate instrument grid (string count)

    // Get active tuning object from activeInstrument
    const activeTuningObj = activeInstrumentObj?.tunings.find((t) => t.name === activeTuningName) || activeInstrumentObj?.tunings[0];

    // Get string count from notes in active tuning object
    const stringCount = activeTuningObj ? activeTuningObj.strings.length : 6;
    // Generate an array based on the string count
    const instrumentStrings = Array.from({ length: stringCount }, (_, i) => i);

    // Safeguard
    if (db.length === 0) return <div>No instruments found in database.</div>;
    return (
        <div className="w-full flex flex-col items-center">

            {/* Dropdown Controls */}
            <InstrumentControls
                instruments={availableInstruments}
                tunings={availableTunings}
                activeInstrument={activeInstrumentName}
                activeTuning={activeTuningName}
                onInstrumentChange={setActiveInstrumentName}
                onTuningChange={setActiveTuningName}
            />

            {/* Instrument Grid */}
            <div className="w-full max-w-6xl mx-auto overflow-x-auto py-8">
                <div className="flex flex-col min-w-max border-y border-l border-slate-700 bg-amber-950/20 shadow-xl rounded-sm">
                    {instrumentStrings.map((stringIndex) => (
                        <InstrumentString
                            key={stringIndex}
                            instrument={activeInstrumentName}
                            tuning={activeTuningName}
                            stringIndex={stringIndex}
                        />
                    ))}
                </div>
            </div>

        </div>
    );
}
