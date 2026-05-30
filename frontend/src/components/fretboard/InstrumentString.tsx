import FretNode from "./FretNode";

interface InstrumentStringProps {
    instrument: string;
    tuning: string;
    stringIndex: number;
    totalFrets?: number;
}

export default function InstrumentString({ instrument, tuning, stringIndex, totalFrets = 24 }: InstrumentStringProps) {
    // Generate an array of frets (including open string)
    const frets = Array.from({ length: totalFrets + 1 }, (_, i) => i);

    return (
        // Map over the frets array and generate FretNodes
        <div className="flex w-full">
            {frets.map((fret) => (<FretNode
                key={fret}
                instrument={instrument}
                tuning={tuning}
                stringIndex={stringIndex}
                fretIndex={fret}
            />

            ))}
        </div>

    );


}
