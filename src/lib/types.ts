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
