import React, { useState } from 'react';
import { ShieldAlert, ShieldCheck, FileText, ChevronDown, ChevronUp } from 'lucide-react';
import { Card } from '../components/common/Card';
import { EmptyState } from '../components/common/EmptyState';
import type { SecurityIncident } from '../types';

interface SecurityPageProps {
  incidents: SecurityIncident[];
  onReviewIncidentActions: (incidentId: string) => void;
}

export const SecurityPage: React.FC<SecurityPageProps> = ({
  incidents,
  onReviewIncidentActions,
}) => {
  const [expandedIncidentId, setExpandedIncidentId] = useState<string | null>(
    incidents.length > 0 ? incidents[0].id : null
  );

  const toggleExpand = (id: string) => {
    setExpandedIncidentId(expandedIncidentId === id ? null : id);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Header Summary */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <div className="text-lg font-bold text-primary">Security &amp; Behavioral Incidents</div>
          <div className="text-xs text-muted">
            Explainable behavioral deviations supported by concrete baseline comparison. TENREC never executes autonomous blocks.
          </div>
        </div>
      </div>

      {/* Incident List */}
      {incidents.length === 0 ? (
        <Card>
          <EmptyState
            title="Zero Security Incidents Active"
            message="All observed network sockets, executable paths, and transfer rates are consistent with machine baseline."
            icon={<ShieldCheck size={36} className="text-green" />}
          />
        </Card>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {incidents.map((incident) => {
            const isExpanded = expandedIncidentId === incident.id;
            return (
              <Card key={incident.id} className="incident-card">
                {/* Header row */}
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'flex-start',
                    cursor: 'pointer',
                  }}
                  onClick={() => toggleExpand(incident.id)}
                >
                  <div style={{ display: 'flex', alignItems: 'flex-start', gap: 10 }}>
                    <div
                      style={{
                        marginTop: 2,
                        padding: 6,
                        borderRadius: 'var(--radius-sm)',
                        backgroundColor:
                          incident.severity === 'CRITICAL' || incident.severity === 'HIGH'
                            ? 'var(--status-red-bg)'
                            : 'var(--status-amber-bg)',
                        color:
                          incident.severity === 'CRITICAL' || incident.severity === 'HIGH'
                            ? 'var(--status-red)'
                            : 'var(--status-amber)',
                      }}
                    >
                      <ShieldAlert size={18} />
                    </div>
                    <div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span
                          className={`badge ${
                            incident.severity === 'CRITICAL' || incident.severity === 'HIGH'
                              ? 'badge-red'
                              : 'badge-amber'
                          }`}
                          style={{ fontSize: '9.5px' }}
                        >
                          {incident.severity}
                        </span>
                        <span className="text-base font-bold text-primary">{incident.title}</span>
                      </div>
                      <div className="text-xs text-secondary" style={{ marginTop: 3 }}>
                        {incident.summary}
                      </div>
                    </div>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                    <div style={{ textAlign: 'right' }}>
                      <div className="mono text-xs font-semibold text-secondary">
                        {incident.confidencePercent}% confidence
                      </div>
                      <div className="mono text-xs text-muted" style={{ fontSize: '10px' }}>
                        Observed: {incident.lastObserved}
                      </div>
                    </div>
                    {isExpanded ? <ChevronUp size={16} className="text-muted" /> : <ChevronDown size={16} className="text-muted" />}
                  </div>
                </div>

                {/* Expanded Details & Evidence Model */}
                {isExpanded && (
                  <div
                    style={{
                      marginTop: 14,
                      paddingTop: 14,
                      borderTop: '1px solid var(--border-subtle)',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 12,
                    }}
                  >
                    {/* Why it was detected */}
                    <div
                      style={{
                        background: 'var(--bg-secondary)',
                        padding: '10px 12px',
                        borderRadius: 'var(--radius-md)',
                        border: '1px solid var(--border-subtle)',
                      }}
                    >
                      <div className="text-xs font-semibold text-secondary" style={{ marginBottom: 4 }}>
                        Why TENREC Flagged This Activity:
                      </div>
                      <div className="text-xs text-muted" style={{ lineHeight: 1.5 }}>
                        {incident.whyDetected}
                      </div>
                      <div className="text-xs text-cyan mono" style={{ marginTop: 4 }}>
                        Baseline Comparison: {incident.baselineComparison}
                      </div>
                    </div>

                    {/* Structured Evidence Grid */}
                    <div>
                      <div
                        className="text-xs font-semibold text-secondary"
                        style={{
                          textTransform: 'uppercase',
                          letterSpacing: '0.04em',
                          marginBottom: 8,
                          display: 'flex',
                          alignItems: 'center',
                          gap: 6,
                        }}
                      >
                        <FileText size={14} className="text-cyan" />
                        Structured Supporting Evidence ({incident.evidence.length} points):
                      </div>

                      <div className="grid-2">
                        {incident.evidence.map((ev, idx) => (
                          <div
                            key={idx}
                            style={{
                              background: 'var(--bg-secondary)',
                              padding: '8px 10px',
                              borderRadius: 'var(--radius-md)',
                              border: '1px solid var(--border-subtle)',
                            }}
                          >
                            <div className="text-xs font-semibold text-primary">{ev.label}</div>
                            <div
                              style={{
                                display: 'flex',
                                justifyContent: 'space-between',
                                fontSize: '11px',
                                marginTop: 4,
                              }}
                            >
                              <span className="mono text-amber">Observed: {ev.observedValue}</span>
                              <span className="mono text-muted">Baseline: {ev.baselineValue}</span>
                            </div>
                            <div className="text-xs text-muted" style={{ marginTop: 4, fontSize: '10.5px' }}>
                              {ev.explanation}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>

                    {/* Affected Application and Destinations */}
                    <div
                      style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        padding: '8px 10px',
                        background: 'var(--bg-secondary)',
                        borderRadius: 'var(--radius-md)',
                      }}
                    >
                      <div className="text-xs text-muted mono">
                        Entity: <strong className="text-primary">{incident.affectedApplication}</strong>{' '}
                        {incident.affectedPid && `(PID ${incident.affectedPid})`} &bull; Destinations:{' '}
                        <strong className="text-secondary">{incident.destinations.join(', ') || 'None'}</strong>
                      </div>

                      <button
                        className="btn btn-primary"
                        style={{ fontSize: '11px', padding: '5px 12px' }}
                        onClick={() => onReviewIncidentActions(incident.id)}
                      >
                        Review Recommended Actions
                      </button>
                    </div>
                  </div>
                )}
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
};
