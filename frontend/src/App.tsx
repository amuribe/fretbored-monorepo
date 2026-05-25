import { useEffect, useState } from 'react';
// Import from rust WASM
import init, { get_instrument_database, get_note_name } from 'core_engine';
// Custom components
import { Fretboard } from "./components/fretboard";

interface Tuning {
    name: string;
    strings: number[];
}

interface Instrument {
    name: string;
    tunings: Tuning[];
}

export default function App() {
    const [db, setDb] = useState<Instrument[]>([]);
    const [testNote, setTestNote] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        async function loadWasm() {
            try {
                // init WASM
                await init();

                // Rust DB query
                const instrumentsData = get_instrument_database() as Instrument[];
                setDb(instrumentsData);

                // Lookup (Guitar, Standard, String 0, Fret 3)
                const note = get_note_name("Guitar", "Standard", 0, 3);
                setTestNote(note ?? "Not found");

                setLoading(false);
            } catch (error) {
                console.error("Failed to load WASM engine:", error);
            }
        }

        loadWasm();
    }, []);

    if (loading) {
        return <div className="p-8 text-gray-400">Initializing Core Engine...</div>;
    }

    return (
        <div className="min-h-screen bg-slate-900 text-slate-100 p-8 flex flex-col items-center">
            <h1 className="text-3xl font-bold mb-8">Fretbored</h1>

            <Fretboard />

        </div>
    );
}
