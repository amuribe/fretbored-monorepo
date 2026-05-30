export interface Tuning {
  name: string;
  strings: number[];
}

export interface Instrument {
  name: string;
  tunings: Tuning[];
}
