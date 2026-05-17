import { get_note_name } from "core_engine";
import { useEffect, useState } from "react";

interface FretNodeProps {
    stringIndex: number;
    fretIndex: number;
}

export default function FretNode({ stringIndex, fretIndex }: FretNodeProps) {
    // Create note name state
    const [noteName, setNoteName] = useState<string | null>(null);
    // Create state for fret being clicked
    const [isToggled, setIsToggled] = useState<boolean>(false);

    useEffect(() => {
        // Guitar and standard are hardcoded for now, will be dynamic after db is updated in dev
        const note = get_note_name("Guitar", "Standard", stringIndex, fretIndex);

        // Update state
        setNoteName(note ?? null);
    }, [stringIndex, fretIndex]);


    return (
        <div className="relative h-12 w-full flex items-center justify-center border-r border-slate-700">
            {/* Fret metal */}
            <div className="absolute left-0 right-0 h-[2px] bg-slate-400 z-0" />
            <button onClick={() => setIsToggled(!isToggled)}
                className={`z-10 w-8 h-8 rounded-full
                    flex items-center justify-center 
                    text-xs font-mono transition-colors
                    ${isToggled ? "bg-emerald-500 text-slate-950 font-bold"
                        : "bg-slate-800 text-slate-300 hover:bg-slate-700"}
                    `}

            >{noteName}</button>
        </div>

    );
}


