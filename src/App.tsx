import React, { useState, useEffect, useCallback } from 'react';
import { Sidebar } from './components/layout/Sidebar';
import type { PageId } from './components/layout/Sidebar';
import { TopBar } from './components/layout/TopBar';
import { OverviewPage } from './pages/OverviewPage';
import { SystemPage } from './pages/SystemPage';
import { NetworkPage } from './pages/NetworkPage';
import { ApplicationsPage } from './pages/ApplicationsPage';
import { ProcessesPage } from './pages/ProcessesPage';
import { SecurityPage } from './pages/SecurityPage';
import { ProtectionPage } from './pages/ProtectionPage';
import { SettingsPage } from './pages/SettingsPage';
import { ActionReviewModal } from './components/common/ActionReviewModal';
import { tenrecApi } from './services/api';
import type {
  SystemSnapshot,
  NetworkThroughput,
  SecurityIncident,
  WhatChangedItem,
  ConnectionInfo,
  ListeningPortInfo,
  TopDestination,
  ProcessItem,
  ApplicationItem,
  ProposedAction,
  ActivePolicy,
  RestorationSnapshot,
  HardwareCapability,
  SystemHealthStatus,
} from './types';

export const App: React.FC = () => {
  const [currentPage, setCurrentPage] = useState<PageId>('overview');

  // Real Telemetry State
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [network, setNetwork] = useState<NetworkThroughput | null>(null);
  const [connections, setConnections] = useState<ConnectionInfo[]>([]);
  const [listeningPorts, setListeningPorts] = useState<ListeningPortInfo[]>([]);
  const [destinations, setDestinations] = useState<TopDestination[]>([]);
  const [processes, setProcesses] = useState<ProcessItem[]>([]);
  const [applications, setApplications] = useState<ApplicationItem[]>([]);
  const [incidents, setIncidents] = useState<SecurityIncident[]>([]);
  const [whatChanged, setWhatChanged] = useState<WhatChangedItem[]>([]);
  const [activePolicies, setActivePolicies] = useState<ActivePolicy[]>([]);
  const [snapshots, setSnapshots] = useState<RestorationSnapshot[]>([]);
  const [capabilities, setCapabilities] = useState<HardwareCapability[]>([]);

  // Historical Telemetry Series (Chronological)
  const [cpuHistory, setCpuHistory] = useState<number[]>([]);
  const [memoryHistory, setMemoryHistory] = useState<number[]>([]);
  const [netThroughputHistory, setNetThroughputHistory] = useState<{ inBytes: number[]; outBytes: number[] }>({
    inBytes: [],
    outBytes: [],
  });

  const [isRefreshing, setIsRefreshing] = useState(false);

  // User Approval Modal State
  const [isActionModalOpen, setIsActionModalOpen] = useState(false);
  const [currentProposedActions, setCurrentProposedActions] = useState<ProposedAction[]>([]);

  // Feedback Notification banner
  const [toastMessage, setToastMessage] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  const showToast = (text: string, type: 'success' | 'error' = 'success') => {
    setToastMessage({ text, type });
    setTimeout(() => setToastMessage(null), 4000);
  };

  // Master telemetry fetcher
  const fetchTelemetry = useCallback(async () => {
    try {
      setIsRefreshing(true);
      const [
        snap,
        net,
        conns,
        ports,
        dests,
        procs,
        apps,
        incs,
        changed,
        policies,
        restores,
        caps,
      ] = await Promise.allSettled([
        tenrecApi.getSystemSnapshot(),
        tenrecApi.getNetworkThroughput(),
        tenrecApi.getActiveConnections(),
        tenrecApi.getListeningPorts(),
        tenrecApi.getTopDestinations(),
        tenrecApi.getProcesses(),
        tenrecApi.getApplications(),
        tenrecApi.getSecurityIncidents(),
        tenrecApi.getWhatChanged(),
        tenrecApi.getActivePolicies(),
        tenrecApi.getRestorationSnapshots(),
        tenrecApi.getHardwareCapabilities(),
      ]);

      if (snap.status === 'fulfilled') {
        const s = snap.value;
        setSnapshot(s);
        setCpuHistory((prev) => [...prev.slice(-30), s.cpu.usagePercent]);
        setMemoryHistory((prev) => [...prev.slice(-30), s.memory.usagePercent]);
      }

      if (net.status === 'fulfilled') {
        const n = net.value;
        setNetwork(n);
        setNetThroughputHistory((prev) => ({
          inBytes: [...prev.inBytes.slice(-30), n.bytesInSec],
          outBytes: [...prev.outBytes.slice(-30), n.bytesOutSec],
        }));
      }

      if (conns.status === 'fulfilled') setConnections(conns.value);
      if (ports.status === 'fulfilled') setListeningPorts(ports.value);
      if (dests.status === 'fulfilled') setDestinations(dests.value);
      if (procs.status === 'fulfilled') setProcesses(procs.value);
      if (apps.status === 'fulfilled') setApplications(apps.value);
      if (incs.status === 'fulfilled') setIncidents(incs.value);
      if (changed.status === 'fulfilled') setWhatChanged(changed.value);
      if (policies.status === 'fulfilled') setActivePolicies(policies.value);
      if (restores.status === 'fulfilled') setSnapshots(restores.value);
      if (caps.status === 'fulfilled') setCapabilities(caps.value);
    } catch (err) {
      console.error('Telemetry refresh error:', err);
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  // Poll loop: Every 2.5s for real-time telemetry
  useEffect(() => {
    fetchTelemetry();
    const interval = setInterval(fetchTelemetry, 2500);
    return () => clearInterval(interval);
  }, [fetchTelemetry]);

  // Overall system health calculation
  const systemStatus: SystemHealthStatus =
    incidents.some((i) => i.severity === 'CRITICAL' && i.status === 'ACTIVE')
      ? 'CRITICAL'
      : incidents.some((i) => (i.severity === 'HIGH' || i.severity === 'MEDIUM') && i.status === 'ACTIVE')
      ? 'ATTENTION'
      : 'HEALTHY';

  // Handler to open ActionReviewModal for a given incident or all
  const handleOpenActionReview = async (incidentId?: string) => {
    try {
      const actions = await tenrecApi.getProposedActions(incidentId);
      setCurrentProposedActions(actions);
      setIsActionModalOpen(true);
    } catch (err) {
      showToast('Failed to retrieve proposed actions from recommendation engine', 'error');
    }
  };

  // Handler when user clicks "Apply Selected Changes" in the ActionReviewModal
  const handleApplyActions = async (selectedActions: ProposedAction[]) => {
    try {
      const result = await tenrecApi.applyApprovedActions(selectedActions);
      if (result.success) {
        showToast(`Successfully applied ${result.appliedCount} approved action(s). Snapshot recorded.`);
        fetchTelemetry();
      } else {
        showToast('Action application failed or was canceled.', 'error');
      }
    } catch (err) {
      showToast('Error applying approved actions.', 'error');
    }
  };

  // Handler for restoring a snapshot
  const handleRestoreSnapshot = async (snapshotId: string) => {
    try {
      const res = await tenrecApi.restoreSnapshot(snapshotId);
      if (res.success) {
        showToast('Snapshot restored successfully.');
        fetchTelemetry();
      } else {
        showToast(res.message || 'Restoration failed.', 'error');
      }
    } catch (err) {
      showToast('Error executing rollback.', 'error');
    }
  };

  // Handler for "Restore Everything"
  const handleRestoreAll = async () => {
    try {
      const res = await tenrecApi.restoreAll();
      if (res.success) {
        showToast(`Restored all systems to normal state (${res.revertedCount} actions rolled back).`);
        fetchTelemetry();
      } else {
        showToast(res.message || 'Restoration failed.', 'error');
      }
    } catch (err) {
      showToast('Error during global restoration.', 'error');
    }
  };

  // Handler when user requests policy from Applications page
  const handleRequestPolicyForApp = async (app: ApplicationItem) => {
    const customAction: ProposedAction = {
      id: `custom_app_policy_${app.id}_${Date.now()}`,
      title: `Create Temporary Network Block for ${app.name}`,
      description: `Add temporary outbound firewall restriction rule scoped to '${app.executablePath}'.`,
      risk: 'SAFE',
      rationale: `User requested restriction for ${app.name} to preserve bandwidth and audit connections.`,
      expectedBenefit: 'Stops background outbound network requests immediately.',
      potentialConsequence: 'Application cannot synchronize or access web endpoints until restored.',
      currentState: 'Outbound Allowed',
      proposedState: 'Temporary Block',
      isReversible: true,
      defaultSelected: true,
      selected: true,
      durationMinutes: 60,
      targetType: 'firewall_rule',
      targetIdentifier: app.name,
    };
    setCurrentProposedActions([customAction]);
    setIsActionModalOpen(true);
  };

  // Handler when user requests process control from Processes page
  const handleRequestProcessControl = async (
    proc: ProcessItem,
    actionType: 'suspend' | 'priority' | 'terminate'
  ) => {
    const isRisky = actionType === 'terminate' || actionType === 'suspend';
    const customAction: ProposedAction = {
      id: `proc_ctrl_${proc.pid}_${actionType}_${Date.now()}`,
      title: `${actionType.toUpperCase()} Process ${proc.name} (PID ${proc.pid})`,
      description: `Perform ${actionType} on process running from '${proc.executablePath}'.`,
      risk: isRisky ? 'RISKY' : 'SAFE',
      rationale: `Targeted process control explicitly invoked by user in Process Explorer.`,
      expectedBenefit: 'Releases processor and memory resources instantly.',
      potentialConsequence: isRisky
        ? 'May cause unsaved user work in the target application to be lost.'
        : 'Slightly reduces execution scheduling frequency.',
      currentState: 'Active',
      proposedState: actionType.toUpperCase(),
      isReversible: actionType !== 'terminate',
      defaultSelected: !isRisky,
      selected: !isRisky,
      durationMinutes: 0,
      targetType: actionType === 'priority' ? 'process_priority' : 'process_suspend',
      targetIdentifier: `${proc.name} (PID ${proc.pid})`,
    };
    setCurrentProposedActions([customAction]);
    setIsActionModalOpen(true);
  };

  return (
    <div className="app-container">
      {/* Navigation Sidebar */}
      <Sidebar
        currentPage={currentPage}
        onSelectPage={setCurrentPage}
        incidentCount={incidents.filter((i) => i.status === 'ACTIVE').length}
        activePolicyCount={activePolicies.filter((p) => p.status === 'ACTIVE').length}
        onRestoreAllClick={handleRestoreAll}
      />

      {/* Main Workspace Area */}
      <div className="main-content">
        <TopBar
          status={systemStatus}
          snapshot={snapshot}
          network={network}
          onRefresh={fetchTelemetry}
          isRefreshing={isRefreshing}
        />

        {/* User Toast Notification Banner */}
        {toastMessage && (
          <div
            style={{
              padding: '8px 20px',
              backgroundColor:
                toastMessage.type === 'success' ? 'var(--status-green-bg)' : 'var(--status-red-bg)',
              borderBottom: `1px solid ${
                toastMessage.type === 'success' ? 'var(--status-green-border)' : 'var(--status-red-border)'
              }`,
              color: toastMessage.type === 'success' ? 'var(--status-green)' : 'var(--status-red)',
              fontSize: '12px',
              fontWeight: 600,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <span>{toastMessage.text}</span>
            <button
              onClick={() => setToastMessage(null)}
              style={{ background: 'none', border: 'none', color: 'inherit', cursor: 'pointer' }}
            >
              &times;
            </button>
          </div>
        )}

        {/* Viewport for Active Page */}
        <main className="page-viewport">
          {currentPage === 'overview' && (
            <OverviewPage
              snapshot={snapshot}
              network={network}
              incidents={incidents}
              whatChanged={whatChanged}
              connections={connections}
              destinations={destinations}
              applications={applications}
              netThroughputHistory={netThroughputHistory}
              cpuHistory={cpuHistory}
              memoryHistory={memoryHistory}
              onOpenActionReview={handleOpenActionReview}
              onNavigate={setCurrentPage}
            />
          )}

          {currentPage === 'system' && <SystemPage snapshot={snapshot} />}

          {currentPage === 'network' && (
            <NetworkPage
              network={network}
              connections={connections}
              listeningPorts={listeningPorts}
              destinations={destinations}
              throughputHistory={netThroughputHistory}
            />
          )}

          {currentPage === 'applications' && (
            <ApplicationsPage
              applications={applications}
              onRequestPolicy={handleRequestPolicyForApp}
            />
          )}

          {currentPage === 'processes' && (
            <ProcessesPage
              processes={processes}
              onRequestProcessControl={handleRequestProcessControl}
            />
          )}

          {currentPage === 'security' && (
            <SecurityPage
              incidents={incidents}
              onReviewIncidentActions={handleOpenActionReview}
            />
          )}

          {currentPage === 'protection' && (
            <ProtectionPage
              policies={activePolicies}
              snapshots={snapshots}
              onRestoreSnapshot={handleRestoreSnapshot}
              onRestoreAll={handleRestoreAll}
            />
          )}

          {currentPage === 'settings' && (
            <SettingsPage
              capabilities={capabilities}
              onRefreshCapabilities={fetchTelemetry}
            />
          )}
        </main>
      </div>

      {/* User Approval & Staging Modal */}
      <ActionReviewModal
        isOpen={isActionModalOpen}
        actions={currentProposedActions}
        onClose={() => setIsActionModalOpen(false)}
        onApply={handleApplyActions}
      />
    </div>
  );
};

export default App;

