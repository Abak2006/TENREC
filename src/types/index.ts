// TENREC - Core Domain Types & Interfaces

export type SystemHealthStatus = 'HEALTHY' | 'ATTENTION' | 'CRITICAL';

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

export interface DiskMetrics {
  drives: DiskDrive[];
  readBytesSec: number;
  writeBytesSec: number;
}

export interface PowerMetrics {
  acConnected: boolean;
  hasBattery: boolean;
  batteryPercent: number | null;
  batteryStatus: string;
  powerMode: string;
}

export interface GpuMetrics {
  name: string;
  isAvailable: boolean;
  memoryTotalBytes: number | null;
  memoryUsedBytes: number | null;
  utilizationPercent: number | null;
  temperatureC: number | null;
  statusMessage: string;
}

export interface SystemSnapshot {
  timestamp: string;
  uptimeSeconds: number;
  hostname: string;
  osName: string;
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  disk: DiskMetrics;
  power: PowerMetrics;
  gpu: GpuMetrics;
}

export interface NetworkThroughput {
  bytesInSec: number;
  bytesOutSec: number;
  totalBytesIn: number;
  totalBytesOut: number;
  latencyMs: number | null;
  packetLossPercent: number | null;
  activeConnectionCount: number;
  primaryInterface: string;
  linkSpeedMbps: number | null;
}

export interface ConnectionInfo {
  id: string;
  pid: number;
  processName: string;
  applicationName: string;
  protocol: 'TCP' | 'UDP';
  localAddress: string;
  localPort: number;
  remoteAddress: string;
  remotePort: number;
  state: string;
  hostname: string;
  bytesSent: number;
  bytesReceived: number;
  firstSeen: string;
  lastSeen: string;
  isUnusual: boolean;
  isNew: boolean;
}

export interface ListeningPortInfo {
  port: number;
  protocol: 'TCP' | 'UDP';
  pid: number;
  processName: string;
  applicationName: string;
  bindAddress: string;
  state: string;
  isKnownService: boolean;
}

export interface TopDestination {
  destination: string;
  hostname: string;
  application: string;
  totalBytes: number;
  connectionCount: number;
  lastSeen: string;
  securityStatus: 'normal' | 'unusual' | 'suspicious';
}

export interface ProcessItem {
  pid: number;
  ppid: number | null;
  name: string;
  executablePath: string;
  publisher: string;
  cpuPercent: number;
  memoryBytes: number;
  memoryPercent: number;
  netUploadBps: number;
  netDownloadBps: number;
  connectionCount: number;
  startTime: string;
  isElevated: boolean;
  applicationName: string;
}

export interface ApplicationItem {
  id: string;
  name: string;
  publisher: string;
  executablePath: string;
  processCount: number;
  pids: number[];
  cpuPercent: number;
  memoryBytes: number;
  netUploadBps: number;
  netDownloadBps: number;
  connectionCount: number;
  firstSeen: string;
  lastSeen: string;
  baselineStatus: 'normal' | 'learning' | 'divergent';
  securityStatus: 'healthy' | 'attention' | 'incident';
}

export type IncidentSeverity = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';

export interface EvidenceItem {
  key: string;
  label: string;
  observedValue: string;
  baselineValue: string;
  explanation: string;
}

export interface SecurityIncident {
  id: string;
  title: string;
  severity: IncidentSeverity;
  confidencePercent: number;
  firstObserved: string;
  lastObserved: string;
  affectedApplication: string;
  affectedPid: number | null;
  destinations: string[];
  summary: string;
  whyDetected: string;
  baselineComparison: string;
  evidence: EvidenceItem[];
  proposedActionIds: string[];
  status: 'ACTIVE' | 'REVIEWED' | 'DISMISSED';
}

export interface WhatChangedItem {
  id: string;
  category: 'new_application' | 'new_destination' | 'traffic_spike' | 'listening_port' | 'anomaly';
  title: string;
  description: string;
  timestamp: string;
  entityName: string;
  severity: 'info' | 'attention' | 'warning';
}

export type ActionRisk = 'SAFE' | 'RISKY';

export interface ProposedAction {
  id: string;
  incidentId?: string;
  title: string;
  description: string;
  risk: ActionRisk;
  rationale: string;
  expectedBenefit: string;
  potentialConsequence: string;
  currentState: string;
  proposedState: string;
  isReversible: boolean;
  defaultSelected: boolean;
  selected: boolean;
  durationMinutes: number; // 0 = until restored manually
  targetType: 'firewall_rule' | 'power_mode' | 'process_priority' | 'process_suspend' | 'network_throttle';
  targetIdentifier: string;
}

export interface RestorationSnapshot {
  id: string;
  timestamp: string;
  title: string;
  actionSummary: string[];
  rulesCreated: string[];
  previousPowerMode?: string;
  suspendedPids?: number[];
  status: 'ACTIVE' | 'RESTORED';
}

export interface ActivePolicy {
  id: string;
  name: string;
  policyType: 'FIREWALL_BLOCK' | 'FIREWALL_RESTRICT' | 'POWER_MODE' | 'PROCESS_PRIORITY';
  applicationName: string;
  destinationEndpoint: string | null;
  createdAt: string;
  expiresAt: string | null;
  reason: string;
  status: 'ACTIVE' | 'EXPIRED' | 'REVERTED';
}

export interface HardwareCapability {
  name: string;
  available: boolean;
  details: string;
  category: 'system' | 'vendor_extension';
}
