interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
    label: string;
    options: string[];
}

export default function Select({ label, options, ...props }: SelectProps) {
    return (
        <div className="flex flex-col">
            <label className="text-xs text-slate-400 font-bold mb-1 uppercase tracking-wider">
                {label}
            </label>
            <select
                className="bg-slate-900 border border-slate-600 rounded px-3 py-2 text-white outline-none focus:border-emerald-500"
                {...props}
            >
                {options.map((opt) => (
                    <option key={opt} value={opt}>
                        {opt}
                    </option>
                ))}
            </select>
        </div>
    );
}
