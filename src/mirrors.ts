const STATUS_API_URL = "https://mirrors.ustc.edu.cn/status/json";

export interface MirrorStatus {
  name: string;
  lastSuccess: number;
}

export interface OutdatedEntry {
  name: string;
}

export class MirrorsStatusChecker {
  private thresholdDays: number;
  private whitelist: Set<string>;

  constructor(thresholdDays: number, whitelist: Set<string>) {
    this.thresholdDays = thresholdDays;
    this.whitelist = whitelist;
  }

  async fetch(): Promise<MirrorStatus[]> {
    const resp = await fetch(STATUS_API_URL);
    if (!resp.ok) {
      throw new Error(
        `USTC API returned HTTP ${resp.status}: ${resp.statusText}`
      );
    }
    return resp.json() as Promise<MirrorStatus[]>;
  }

  findOutdated(entries: MirrorStatus[]): OutdatedEntry[] {
    const cutoff = Date.now() - this.thresholdDays * 86400 * 1000;

    return entries
      .filter((entry) => {
        if (this.whitelist.has(entry.name)) return false;
        const lastSyncMs = entry.lastSuccess * 1000;
        return lastSyncMs <= 0 || lastSyncMs < cutoff;
      })
      .map((entry) => ({ name: entry.name }));
  }

  async check(): Promise<OutdatedEntry[]> {
    const entries = await this.fetch();
    return this.findOutdated(entries);
  }
}
