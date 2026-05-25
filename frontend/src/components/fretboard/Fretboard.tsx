import InstrumentString from "./InstrumentString";

export default function Fretboard() {
    // String count is hardcoded right now but will be a state(from rust engine) later on in development
    const stringCount = 6;

    // Generate an array based on the string count
    const instrumentStrings = Array.from({ length: stringCount }, (_, i) => i);

    return (
        <div className="w-full max-w-6xl mx-auto overflow-x-auto py-8">
            <div className="flex flex-col min-w-max 
            border-y border-l border-slate-700 
            bg-amber-950/20 shadow-xl rounded-sm"
            >
                {instrumentStrings.map((stringIndex) => (
                    <InstrumentString
                        key={stringIndex}
                        stringIndex={stringIndex}
                    />
                ))}
            </div>
        </div>
    );

}
