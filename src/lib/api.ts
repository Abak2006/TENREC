// Typed wrappers around the Tauri IPC commands in src-tauri/src/ipc/commands.rs.
import type {
  ActivePolicy, AppStatus, ApplicationItem, ApplyResult, AuditEntry, ConnectionInfo, DatabaseStats, FocusOptions, FocusPreview, FocusStatus, PrivacyEvidence, PrivacySetting, ScheduledTaskItem, SettingChangeResult, StartupItem,
  HardwareCapabilityItem, HardwareDiagnostics, ListeningPortInfo, NetworkThroughput, ProcessItem,
  ProposedAction, RestorationSnapshot, RestoreResult, SecurityIncident, SystemSnapshot, TopDestination,
  WhatChangedItem,
} from './types';

export const isTauri = (): boolean => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export class NotInTauriError extends Error {
  constructor() {
    super('TENREC must run as the desktop app to read system data.');
  }
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new NotInTauriError();
  const { invoke } = await import('@tauri-apps/api/core');
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    // Commands return plain string errors written for the user.
    throw new Error(typeof err === 'string' ? err : err instanceof Error ? err.message : `Command ${cmd} failed`);
  }
}

export const api = {
  status: () => call<AppStatus>('get_status'),
  systemSnapshot: () => call<SystemSnapshot | null>('get_system_snapshot'),
  throughput: () => call<NetworkThroughput | null>('get_network_throughput'),
  connections: () => call<ConnectionInfo[]>('get_active_connections'),
  listeningPorts: () => call<ListeningPortInfo[]>('get_listening_ports'),
  destinations: () => call<TopDestination[]>('get_top_destinations'),
  processes: () => call<ProcessItem[]>('get_processes'),
  applications: () => call<ApplicationItem[]>('get_applications'),
  incidents: () => call<SecurityIncident[]>('get_security_incidents'),
  setIncidentStatus: (incidentId: string, status: 'ACTIVE' | 'DISMISSED') =>
    call<void>('set_incident_status', { incidentId, status }),
  whatChanged: () => call<WhatChangedItem[]>('get_what_changed'),
  proposedActions: (incidentId?: string) => call<ProposedAction[]>('get_proposed_actions', { incidentId }),
  prepareProcessAction: (pid: number, kind: 'priority' | 'suspend') =>
    call<ProposedAction>('prepare_process_action', { pid, kind }),
  prepareAppBlock: (appId: string) => call<ProposedAction>('prepare_app_block', { appId }),
  applyActions: (actionIds: string[]) => call<ApplyResult>('apply_actions', { actionIds }),
  activePolicies: () => call<ActivePolicy[]>('get_active_policies'),
  snapshots: () => call<RestorationSnapshot[]>('get_restoration_snapshots'),
  restoreSnapshot: (snapshotId: string) => call<RestoreResult>('restore_snapshot', { snapshotId }),
  restoreAll: () => call<RestoreResult>('restore_all'),
  diagnostics: () => call<HardwareDiagnostics>('get_hardware_diagnostics'),
  capabilities: () => call<HardwareCapabilityItem[]>('get_hardware_registry'),
  auditLog: () => call<AuditEntry[]>('get_audit_log'),
  databaseStats: () => call<DatabaseStats>('get_database_stats'),
  focusStatus: () => call<FocusStatus>('get_focus_status'),
  previewFocus: (appId: string, options: FocusOptions) => call<FocusPreview>('preview_focus', { appId, options }),
  startFocus: (appId: string, options: FocusOptions) => call<FocusStatus>('start_focus', { appId, options }),
  stopFocus: () => call<RestoreResult>('stop_focus'),
  startupItems: () => call<StartupItem[]>('get_startup_items'),
  setStartupEnabled: (itemId: string, enabled: boolean) => call<SettingChangeResult>('set_startup_enabled', { itemId, enabled }),
  scheduledTasks: () => call<ScheduledTaskItem[]>('get_scheduled_tasks'),
  setTaskEnabled: (path: string, enabled: boolean) => call<SettingChangeResult>('set_task_enabled', { path, enabled }),
  privacySettings: () => call<PrivacySetting[]>('get_privacy_settings'),
  setPrivacySetting: (id: string, isPrivate: boolean) => call<SettingChangeResult>('set_privacy_setting', { id, private: isPrivate }),
  privacyEvidence: () => call<PrivacyEvidence>('get_privacy_evidence'),
  undoSnapshot: (snapshotId: string) => call<RestoreResult>('restore_snapshot', { snapshotId }),
  purgeTelemetry: (retentionHours: number) => call<number>('purge_old_telemetry', { retentionHours }),
};
