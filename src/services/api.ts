// TENREC - Tauri IPC & Native Service Client
import type {
  SystemSnapshot,
  NetworkThroughput,
  ConnectionInfo,
  ListeningPortInfo,
  TopDestination,
  ProcessItem,
  ApplicationItem,
  SecurityIncident,
  WhatChangedItem,
  ProposedAction,
  ActivePolicy,
  RestorationSnapshot,
  HardwareCapability,
} from '../types';

// Safely detect if running inside Tauri
const isTauriEnv = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

// Generic safe invoke wrapper
async function invokeTauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriEnv()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(cmd, args);
    } catch (err) {
      console.error(`[TENREC Native Error] command '${cmd}' failed:`, err);
      throw err;
    }
  } else {
    // If running in browser preview during UI development, provide honest empty / diagnostic state
    throw new Error(`Native Windows telemetry unavailable outside Tauri runtime.`);
  }
}

export const tenrecApi = {
  // System Telemetry
  getSystemSnapshot: async (): Promise<SystemSnapshot> => {
    return await invokeTauri<SystemSnapshot>('get_system_snapshot');
  },

  // Network Telemetry
  getNetworkThroughput: async (): Promise<NetworkThroughput> => {
    return await invokeTauri<NetworkThroughput>('get_network_throughput');
  },

  getActiveConnections: async (): Promise<ConnectionInfo[]> => {
    return await invokeTauri<ConnectionInfo[]>('get_active_connections');
  },

  getListeningPorts: async (): Promise<ListeningPortInfo[]> => {
    return await invokeTauri<ListeningPortInfo[]>('get_listening_ports');
  },

  getTopDestinations: async (): Promise<TopDestination[]> => {
    return await invokeTauri<TopDestination[]>('get_top_destinations');
  },

  // Process & Application Telemetry
  getProcesses: async (): Promise<ProcessItem[]> => {
    return await invokeTauri<ProcessItem[]>('get_processes');
  },

  getApplications: async (): Promise<ApplicationItem[]> => {
    return await invokeTauri<ApplicationItem[]>('get_applications');
  },

  // Security & Incidents
  getSecurityIncidents: async (): Promise<SecurityIncident[]> => {
    return await invokeTauri<SecurityIncident[]>('get_security_incidents');
  },

  getWhatChanged: async (): Promise<WhatChangedItem[]> => {
    return await invokeTauri<WhatChangedItem[]>('get_what_changed');
  },

  // Protection, Recommendations, Approval & Restoration
  getProposedActions: async (incidentId?: string): Promise<ProposedAction[]> => {
    return await invokeTauri<ProposedAction[]>('get_proposed_actions', { incidentId });
  },

  getActivePolicies: async (): Promise<ActivePolicy[]> => {
    return await invokeTauri<ActivePolicy[]>('get_active_policies');
  },

  getRestorationSnapshots: async (): Promise<RestorationSnapshot[]> => {
    return await invokeTauri<RestorationSnapshot[]>('get_restoration_snapshots');
  },

  getHardwareCapabilities: async (): Promise<HardwareCapability[]> => {
    return await invokeTauri<HardwareCapability[]>('get_hardware_capabilities');
  },

  applyApprovedActions: async (actions: ProposedAction[]): Promise<{ success: boolean; appliedCount: number; snapshotId: string }> => {
    return await invokeTauri<{ success: boolean; appliedCount: number; snapshotId: string }>('apply_approved_actions', { actions });
  },

  restoreSnapshot: async (snapshotId: string): Promise<{ success: boolean; message: string }> => {
    return await invokeTauri<{ success: boolean; message: string }>('restore_snapshot', { snapshotId });
  },

  restoreAll: async (): Promise<{ success: boolean; revertedCount: number; message: string }> => {
    return await invokeTauri<{ success: boolean; revertedCount: number; message: string }>('restore_all');
  },
};
