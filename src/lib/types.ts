export type DbStatus = {
  dbPath: string;
  gamesCount: number;
  playsCount: number;
  pitchesCount: number;
  teamsCount: number;
  playersCount: number;
};

export type ImportProgress = {
  current: number;
  total: number;
  gamePk: number;
  message: string;
};

export type ImportResult = {
  gamesImported: number;
  playsInserted: number;
  pitchesInserted: number;
  gamesSkipped: number;
};

export type StateExpectedRuns = {
  state: string;
  label: string;
  expectedRuns: number;
  frequency: number;
};

export type OffenseBundle = {
  season: number;
  teamId: number | null;
  states: string[];
  matrix: number[][];
  expectedRuns: StateExpectedRuns[];
};

export type TeamOption = {
  teamId: number;
  name: string;
  abbreviation: string;
};

export type RunComparison = {
  state: string;
  label: string;
  teamER: number;
  leagueER: number;
  delta: number;
  frequency: number;
};

export type MomentumBundle = {
  season: number;
  teamId: number | null;
  states: string[];
  coldExpectedRuns: StateExpectedRuns[];
  hotExpectedRuns: StateExpectedRuns[];
  coldMatrix: number[][];
  hotMatrix: number[][];
  coldTotalPlays: number;
  hotTotalPlays: number;
  verdict: string;
  overallDelta: number;
};

export type PitcherSearchResult = {
  playerId: number;
  fullName: string;
  team: string;
  totalPitches: number;
};

export type PitchMatrixData = {
  types: string[];
  matrix: number[][];
  occurrences: number[][];
};

export type CountEntropyData = {
  countState: string;
  entropy: number;
  pitches: number;
};

export type PitchSequenceBundle = {
  pitcherId: number;
  pitcherName: string;
  season: number;
  pitchTypes: string[];
  overallMatrix: PitchMatrixData;
  byCount: Record<string, PitchMatrixData>;
  overallEntropy: number;
  totalPitches: number;
  countEntropy: CountEntropyData[];
};
