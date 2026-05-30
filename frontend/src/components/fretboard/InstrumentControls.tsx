import Select from "../ui/Select";

interface InstrumentControlProps {
    instruments: string[];
    tunings: string[];
    // Current selection
    activeInstrument: string;
    activeTuning: string;

    onInstrumentChange: (instrument: string) => void;
    onTuningChange: (tuning: string) => void;
}

export default function InstrumentControls({
    instruments,
    tunings,
    activeInstrument,
    activeTuning,
    onInstrumentChange,
    onTuningChange,
}: InstrumentControlProps) {
    return (
        <div className="flex gap-4 mb-6 bg-slate-800 p-4 rounded-lg border border-slate-700 w-full max-w-6xl mx-auto">
            <Select
                label="Instrument"
                options={instruments}
                value={activeInstrument}
                onChange={(e) => onInstrumentChange(e.target.value)}
            />
            <Select
                label="Tuning"
                options={tunings}
                value={activeTuning}
                onChange={(e) => onTuningChange(e.target.value)}
            />
        </div>
    )

}
