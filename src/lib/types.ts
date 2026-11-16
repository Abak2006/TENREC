// Mirrors the serde types in src-tauri/src. Keep in sync when changing either side.

export interface AppStatus {
  version: string;
  elevated: boolean;
  sampleSequence: number;
  sampleIntervalMs: number;
  lastError: string | null;
  storageError: string | null;
}

export interface CpuMetrics {
  usagePercent: number;
  frequencyMhz: number | null;
  coreCount: number;
  logicalCoreCount: number;
  perCoreUsage: number[];
  temperatureC: number | null;
}

export interface MemoryMetrics {
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
  usagePercent: number;
}

export interface DiskDrive {
  mountPoint: string;
  volumeName: string;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
  usagePercent: number;
  fileSystem: string;
}

export interface PowerMetrics {
  acConnected: boolean;
  hasBattery: boolean;
  batteryPercent: number | null;
  batteryStatus: string;
}

export interface GpuMetrics {
  name: string;
  isAvailable: boolean;
  statusMessage: string;
}

export interface SystemSnapshot {
  timestamp: string;
  uptimeSeconds: number;
  hostname: string;
  osName: string;
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  disk: { drives: DiskDrive[] };
  io: { readBytesSec: number; writeBytesSec: number };
  power: PowerMetrics;
  gpu: GpuMetrics;
}

export interface NetworkThroughput {
  bytesInSec: number;
  bytesOutSec: number;
  totalBytesIn: number;
  totalBytesOut: number;
  activeConnectionCount: number;
  busiestInterface: string | null;
}

export interface ConnectionInfo {
  id: string;
  pid: number;
  processName: string;
  applicationName: string;
  protocol: string;
  ipVersion: 4 | 6;
  localAddress: string;
  localPort: number;
  remoteAddress: string;
  remotePort: number;
  state: string;
  isPublicRemote: boolean;
  isNew: boolean;
}

export interface ListeningPortInfo {
  port: number;
  protocol: 'TCP' | 'UDP';
  ipVersion: 4 | 6;
  pid: number;
  processName: string;
  applicationName: string;
  bindAddress: string;
  allInterfaces: boolean;
}

export interface TopDestination {
  destination: string;
  applications: string[];
  openConnections: number;
  timesSeen: number;
  firstSeen: string;
  lastSeen: string;
}

export interface ProcessItem {
  pid: number;
  ppid: number | null;
  name: string;
  executablePath: string;
  company: string | null;
  applicationId: string;
  applicationName: string;
  cpuPercent: number;
  memoryBytes: number;
  memoryPercent: number;
  ioReadBps: number;
  ioWriteBps: number;
  connectionCount: number;
  startTime: string;
  runsFromUserWritableLocation: boolean;
  restricted: boolean;
}

export interface ApplicationItem {
  id: string;
  name: string;
  company: string | null;
  executablePath: string;
  processCount: number;
  pids: number[];
  cpuPercent: number;
  memoryBytes: number;
  ioReadBps: number;
  ioWriteBps: number;
  connectionCount: number;
  restrictedProcessCount: number;
  firstSeen: string;
  baselineStatus: 'learning' | 'normal' | 'divergent';
}

export type Severity = 'LOW' | 'MEDIUM' | 'HIGH';

export interface SecurityIncident {
  id: string;
  detector: string;
  title: string;
  severity: Severity;
  firstObserved: string;
  lastObserved: string;
  observationCount: number;
  currentlyObserved: boolean;
  affectedApplication: string;
  affectedPid: number | null;
  summary: string;
  whyDetected: string;
  evidence: { label: string; value: string }[];
  proposedActionIds: string[];
  status: 'ACTIVE' | 'DISMISSED';
}

export type ActionTarget =
  | { kind: 'block_remote_ip'; ip: string }
  | { kind: 'block_program'; path: string }
  | { kind: 'block_inbound_port'; port: number; protocol: string }
  | { kind: 'lower_priority'; pid: number; processName: string; creationTime: number }
  | { kind: 'suspend_process'; pid: number; processName: string; creationTime: number };

export interface ProposedAction {
  id: string;
  incidentId: string | null;
  title: string;
  description: string;
  risk: 'SAFE' | 'RISKY';
  expectedBenefit: string;
  potentialConsequence: string;
  requiresAdmin: boolean;
  defaultSelected: boolean;
  target: ActionTarget;
}

export interface WhatChangedItem {
  id: string;
  category: 'new_application' | 'new_destination' | 'listening_port';
  title: string;
  description: string;
  timestamp: string;
  entityName: string;
  severity: 'info' | 'attention';
}

export interface ChangeRecord {
  change: { kind: ChangeKind };
  description: string;
  status: 'ACTIVE' | 'RESTORED' | 'RESTORE_FAILED';
  note: string | null;
}

export interface RestorationSnapshot {
  id: string;
  timestamp: string;
  title: string;
  changes: ChangeRecord[];
  status: 'ACTIVE' | 'RESTORED' | 'PARTIALLY_RESTORED';
  kind: 'actions' | 'focus' | 'settings';
}

export type ChangeKind = 'firewall_rule' | 'priority' | 'suspended' | 'tuning' | 'setting' | 'scheduled_task' | 'qos_policy' | 'power_mode';

export interface ActivePolicy {
  id: string;
  snapshotId: string;
  kind: ChangeKind;
  description: string;
  createdAt: string;
}

export interface ActionOutcome {
  actionId: string;
  title: string;
  success: boolean;
  error: string | null;
}

export interface ApplyResult {
  snapshotId: string | null;
  outcomes: ActionOutcome[];
}

export interface RestoreResult {
  restored: number;
  failed: number;
  messages: string[];
}

export interface HardwareDiagnostics {
  manufacturer: string;
  model: string;
  cpuName: string;
  cpuCores: number;
  cpuLogicalCores: number;
  gpuName: string;
  gpuDriverVersion: string | null;
  totalMemoryBytes: number;
  osBuild: string;
  isLaptop: boolean;
  hasBattery: boolean;
}

export interface HardwareCapabilityItem {
  id: string;
  name: string;
  category: string;
  provider: string;
  state: 'AVAILABLE' | 'REQUIRES_ADMIN' | 'NOT_PRESENT' | 'NOT_IMPLEMENTED';
  accessMode: 'READ_ONLY' | 'READ_WRITE';
  description: string;
}

export interface AuditEntry {
  id: number;
  timestamp: string;
  actor: string;
  actionType: string;
  target: string;
  details: string;
  success: boolean;
}

export interface DatabaseStats {
  sampleCount: number;
  snapshotCount: number;
  auditCount: number;
  fileSizeBytes: number;
  dbPath: string;
}

export interface FocusOptions {
  efficiencyMode: boolean;
  lowerPriority: boolean;
  lowIo: boolean;
  lowMemory: boolean;
  efficiencyCores: boolean;
  preferGpu: boolean;
  exclusions: string[];
}

export interface FocusPreview {
  appId: string;
  appName: string;
  executablePath: string;
  focusProcessCount: number;
  backgroundProcessCount: number;
  skippedProtected: number;
  skippedRestricted: number;
  skippedExcluded: number;
  hybridCpu: boolean;
  efficiencyCoreCount: number;
}

export interface FocusStatus {
  active: boolean;
  snapshotId: string | null;
  appId: string | null;
  appName: string | null;
  startedAt: string | null;
  options: FocusOptions | null;
  tunedProcesses: number;
  failedProcesses: number;
  gpuPreferenceSet: boolean;
  stats: {
    samples: number;
    backgroundCpuBefore: number | null;
    backgroundCpuDuring: number | null;
    focusCpuDuring: number | null;
  };
  hybridCpu: boolean;
}

export interface StartupItem {
  id: string;
  name: string;
  command: string;
  executablePath: string | null;
  company: string | null;
  source: string;
  sourceLabel: string;
  enabled: boolean;
  requiresAdmin: boolean;
  runningNow: boolean;
}

export interface ScheduledTaskItem {
  path: string;
  name: string;
  author: string | null;
  state: string;
  enabled: boolean;
  triggers: string[];
  actions: string[];
  runsAtLogonOrBoot: boolean;
}

export interface SettingChangeResult {
  snapshotId: string;
  description: string;
}
