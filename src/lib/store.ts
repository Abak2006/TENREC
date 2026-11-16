// Polls the backend and exposes one immutable state object to React.
//
// Live data is read every sample interval (the backend samples on its own thread, so
// these reads are cheap). Slow-changing data is read every 15 s or on demand after
// an action. Polling pauses while the window is hidden.
import { useSyncExternalStore } from 'react';
import { api, NotInTauriError } from './api';
import type {
  ActivePolicy, AppStatus, ApplicationItem, AuditEntry, ConnectionInfo, HardwareCapabilityItem,
  HardwareDiagnostics, ListeningPortInfo, NetworkThroughput, ProcessItem, RestorationSnapshot,
  SecurityIncident, SystemSnapshot, TopDestination, WhatChangedItem,
} from './types';

const HISTORY = 60;

export interface History {
  cpu: number[];
  memory: number[];
  netIn: number[];
  netOut: number[];
}

export interface AppData {
  /** True until the first successful live read. */
  loading: boolean;
  /** Message when the backend cannot be reached at all. */
  error: string | null;
  lastUpdated: number | null;
  status: AppStatus | null;
  snapshot: SystemSnapshot | null;
  throughput: NetworkThroughput | null;
  processes: ProcessItem[];
  applications: ApplicationItem[];
  connections: ConnectionInfo[];
  listening: ListeningPortInfo[];
  destinations: TopDestination[];
  incidents: SecurityIncident[];
  changes: WhatChangedItem[];
  policies: ActivePolicy[];
  snapshots: RestorationSnapshot[];
  audit: AuditEntry[];
  diagnostics: HardwareDiagnostics | null;
  capabilities: HardwareCapabilityItem[];
  history: History;
}

const initial: AppData = {
  loading: true,
  error: null,
  lastUpdated: null,
  status: null,
  snapshot: null,
  throughput: null,
  processes: [],
  applications: [],
  connections: [],
  listening: [],
  destinations: [],
  incidents: [],
  changes: [],
  policies: [],
  snapshots: [],
  audit: [],
  diagnostics: null,
  capabilities: [],
  history: { cpu: [], memory: [], netIn: [], netOut: [] },
};

const push = (arr: number[], v: number) => (arr.length >= HISTORY ? [...arr.slice(1), v] : [...arr, v]);

class Store {
  private state: AppData = initial;
  private listeners = new Set<() => void>();
  private liveTimer: ReturnType<typeof setTimeout> | null = null;
  private lastSequence = -1;
  private failures = 0;
  private started = false;

  get = () => this.state;

  subscribe = (fn: () => void) => {
    this.listeners.add(fn);
    if (!this.started) this.start();
    return () => {
      this.listeners.delete(fn);
    };
  };

  private set(patch: Partial<AppData>) {
    this.state = { ...this.state, ...patch };
    this.listeners.forEach((l) => l());
  }

  private start() {
    this.started = true;
    document.addEventListener('visibilitychange', () => {
      if (!document.hidden) void this.refreshLive();
    });
    void this.refreshLive();
    void this.refreshSlow();
    setInterval(() => {
      if (!document.hidden) void this.refreshSlow();
    }, 15000);
  }

  private schedule(ms: number) {
    if (this.liveTimer) clearTimeout(this.liveTimer);
    this.liveTimer = setTimeout(() => {
      if (document.hidden) this.schedule(ms);
      else void this.refreshLive();
    }, ms);
  }

  refreshLive = async () => {
    const interval = this.state.status?.sampleIntervalMs ?? 2000;
    try {
      const status = await api.status();
      // Skip the heavy reads when the backend has not produced a new sample yet.
      if (status.sampleSequence === this.lastSequence && !this.state.loading) {
        this.set({ status });
        return;
      }
      const [snapshot, throughput, processes, applications, connections, listening, destinations, incidents, changes] =
        await Promise.all([
          api.systemSnapshot(), api.throughput(), api.processes(), api.applications(), api.connections(),
          api.listeningPorts(), api.destinations(), api.incidents(), api.whatChanged(),
        ]);
      this.lastSequence = status.sampleSequence;
      this.failures = 0;
      const h = this.state.history;
      const history = snapshot && throughput
        ? {
            cpu: push(h.cpu, snapshot.cpu.usagePercent),
            memory: push(h.memory, snapshot.memory.usagePercent),
            netIn: push(h.netIn, throughput.bytesInSec),
            netOut: push(h.netOut, throughput.bytesOutSec),
          }
        : h;
      this.set({
        // The first sample takes ~2 s after launch; stay in loading until it exists.
        loading: snapshot === null,
        error: null,
        lastUpdated: Date.now(),
        status, snapshot, throughput, processes, applications, connections, listening, destinations, incidents, changes,
        history,
      });
    } catch (err) {
      this.failures += 1;
      const message = err instanceof NotInTauriError ? err.message : `Lost contact with the TENREC backend: ${(err as Error).message}`;
      if (this.failures >= 2 || err instanceof NotInTauriError) this.set({ error: message, loading: false });
    } finally {
      this.schedule(interval);
    }
  };

  refreshSlow = async () => {
    const results = await Promise.allSettled([
      api.activePolicies(), api.snapshots(), api.auditLog(),
      this.state.diagnostics ? Promise.resolve(this.state.diagnostics) : api.diagnostics(),
      api.capabilities(),
    ]);
    const [policies, snapshots, audit, diagnostics, capabilities] = results;
    this.set({
      policies: policies.status === 'fulfilled' ? policies.value : this.state.policies,
      snapshots: snapshots.status === 'fulfilled' ? snapshots.value : this.state.snapshots,
      audit: audit.status === 'fulfilled' ? audit.value : this.state.audit,
      diagnostics: diagnostics.status === 'fulfilled' ? diagnostics.value : this.state.diagnostics,
      capabilities: capabilities.status === 'fulfilled' ? capabilities.value : this.state.capabilities,
    });
  };

  /** After an action: refresh protection state and incidents right away. */
  refreshAfterAction = async () => {
    await Promise.all([this.refreshSlow(), this.refreshLive()]);
  };
}

export const store = new Store();

export function useAppData<T>(select: (s: AppData) => T): T {
  return useSyncExternalStore(store.subscribe, () => select(store.get()));
}
