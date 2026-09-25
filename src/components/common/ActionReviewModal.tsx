import React, { useState } from 'react';
import { ShieldAlert, ShieldCheck, Check, AlertTriangle, X } from 'lucide-react';
import type { ProposedAction } from '../../types';

interface ActionReviewModalProps {
  isOpen: boolean;
  actions: ProposedAction[];
  onClose: () => void;
  onApply: (selectedActions: ProposedAction[]) => void;
}

export const ActionReviewModal: React.FC<ActionReviewModalProps> = ({
  isOpen,
  actions,
  onClose,
  onApply,
}) => {
  // Preset filter state
  const [preset, setPreset] = useState<'conservative' | 'balanced' | 'advanced'>('conservative');

  // Maintain local selection state initialized with defaultSelected (safe = true, risky = false)
  const [selectedIds, setSelectedIds] = useState<Record<string, boolean>>(() => {
    const initial: Record<string, boolean> = {};
    actions.forEach((a) => {
      // RISKY actions are NEVER selected by default
      initial[a.id] = a.risk === 'SAFE' && a.defaultSelected;
    });
    return initial;
  });

  const [confirmStep, setConfirmStep] = useState(false);

  if (!isOpen) return null;

  const toggleAction = (id: string) => {
    setSelectedIds((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const applyPreset = (p: 'conservative' | 'balanced' | 'advanced') => {
    setPreset(p);
    const updated: Record<string, boolean> = {};
    actions.forEach((a) => {
      if (p === 'conservative') {
        updated[a.id] = a.risk === 'SAFE';
      } else if (p === 'balanced') {
        updated[a.id] = a.risk === 'SAFE';
      } else {
        // Even in advanced mode, user can see them, but risky are still NOT auto-checked!
        updated[a.id] = a.risk === 'SAFE';
      }
    });
    setSelectedIds(updated);
  };

  const safeActions = actions.filter((a) => a.risk === 'SAFE');
  const riskyActions = actions.filter((a) => a.risk === 'RISKY');

  const selectedCount = Object.values(selectedIds).filter(Boolean).length;
  const selectedActionList = actions.filter((a) => selectedIds[a.id]);

  const handleApplyClick = () => {
    if (selectedCount === 0) return;
    setConfirmStep(true);
  };

  const handleFinalConfirm = () => {
    onApply(selectedActionList);
    setConfirmStep(false);
    onClose();
  };

  return (
    <div className="modal-backdrop">
      <div className="modal-content" style={{ width: '740px' }}>
        <div className="modal-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <ShieldAlert size={20} className="text-amber" />
            <div>
              <div className="text-base font-bold">Review System Changes</div>
              <div className="text-xs text-muted">
                TENREC never executes changes autonomously. Review and approve each action individually.
              </div>
            </div>
          </div>
          <button className="btn btn-secondary" style={{ padding: 4 }} onClick={onClose}>
            <X size={16} />
          </button>
        </div>

        <div className="modal-body">
          {/* Preset Selector */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '8px 12px',
              backgroundColor: 'var(--bg-card)',
              borderRadius: 'var(--radius-md)',
              border: '1px solid var(--border-subtle)',
              marginBottom: 16,
            }}
          >
            <span className="text-xs font-semibold text-secondary">Preset:</span>
            <div style={{ display: 'flex', gap: 6 }}>
              {(['conservative', 'balanced', 'advanced'] as const).map((p) => (
                <button
                  key={p}
                  className="btn"
                  style={{
                    padding: '4px 10px',
                    fontSize: '11px',
                    backgroundColor: preset === p ? 'var(--accent-blue)' : 'var(--bg-elevated)',
                    color: preset === p ? '#fff' : 'var(--text-secondary)',
                    border: `1px solid ${preset === p ? 'var(--accent-blue)' : 'var(--border-subtle)'}`,
                  }}
                  onClick={() => applyPreset(p)}
                >
                  {p.toUpperCase()}
                </button>
              ))}
            </div>
          </div>

          {/* Confirmation Step View */}
          {confirmStep ? (
            <div
              style={{
                backgroundColor: 'rgba(239, 68, 68, 0.08)',
                border: '1px solid var(--status-red-border)',
                borderRadius: 'var(--radius-lg)',
                padding: '16px',
                marginBottom: 12,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                <AlertTriangle size={18} className="text-red" />
                <span className="text-sm font-bold text-red">Confirm System Modifications</span>
              </div>
              <div className="text-xs text-secondary" style={{ marginBottom: 12 }}>
                You are about to execute <strong>{selectedCount} approved action(s)</strong>. A pre-state restoration
                snapshot will be recorded automatically before applying:
              </div>
              <ul style={{ paddingLeft: 18, fontSize: '12px', color: 'var(--text-primary)' }}>
                {selectedActionList.map((a) => (
                  <li key={a.id} style={{ marginBottom: 4 }}>
                    <strong>[{a.risk}]</strong> {a.title} &mdash;{' '}
                    <span className="text-muted">{a.proposedState}</span>
                  </li>
                ))}
              </ul>
            </div>
          ) : (
            <>
              {/* Safe Actions Section */}
              <div style={{ marginBottom: 18 }}>
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 6,
                    fontSize: '12px',
                    fontWeight: 700,
                    color: 'var(--status-green)',
                    textTransform: 'uppercase',
                    letterSpacing: '0.04em',
                    marginBottom: 8,
                  }}
                >
                  <ShieldCheck size={16} />
                  Safe Actions ({safeActions.length})
                </div>
                {safeActions.length === 0 ? (
                  <div className="text-xs text-muted">No safe actions proposed for this incident.</div>
                ) : (
                  safeActions.map((action) => (
                    <ActionItemRow
                      key={action.id}
                      action={action}
                      checked={!!selectedIds[action.id]}
                      onToggle={() => toggleAction(action.id)}
                    />
                  ))
                )}
              </div>

              {/* Risky Actions Section */}
              <div>
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 6,
                    fontSize: '12px',
                    fontWeight: 700,
                    color: 'var(--status-red)',
                    textTransform: 'uppercase',
                    letterSpacing: '0.04em',
                    marginBottom: 8,
                  }}
                >
                  <AlertTriangle size={16} />
                  Risky Actions ({riskyActions.length}) &mdash;{' '}
                  <span className="text-xs text-muted" style={{ textTransform: 'none', fontWeight: 400 }}>
                    Unselected by default. May interrupt running processes.
                  </span>
                </div>
                {riskyActions.length === 0 ? (
                  <div className="text-xs text-muted">No risky actions proposed.</div>
                ) : (
                  riskyActions.map((action) => (
                    <ActionItemRow
                      key={action.id}
                      action={action}
                      checked={!!selectedIds[action.id]}
                      onToggle={() => toggleAction(action.id)}
                    />
                  ))
                )}
              </div>
            </>
          )}
        </div>

        <div className="modal-footer">
          {confirmStep ? (
            <>
              <button className="btn btn-secondary" onClick={() => setConfirmStep(false)}>
                Back to Edit
              </button>
              <button className="btn btn-danger" onClick={handleFinalConfirm}>
                Confirm & Apply ({selectedCount})
              </button>
            </>
          ) : (
            <>
              <button className="btn btn-secondary" onClick={onClose}>
                Cancel
              </button>
              <button
                className="btn btn-primary"
                disabled={selectedCount === 0}
                onClick={handleApplyClick}
                style={{ opacity: selectedCount === 0 ? 0.5 : 1 }}
              >
                Apply Selected Changes ({selectedCount})
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

interface ActionItemRowProps {
  action: ProposedAction;
  checked: boolean;
  onToggle: () => void;
}

const ActionItemRow: React.FC<ActionItemRowProps> = ({ action, checked, onToggle }) => {
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      style={{
        backgroundColor: 'var(--bg-card)',
        border: `1px solid ${checked ? 'var(--border-active)' : 'var(--border-subtle)'}`,
        borderRadius: 'var(--radius-md)',
        padding: '10px 12px',
        marginBottom: 8,
        transition: 'border-color 0.15s ease',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 10 }}>
        <div style={{ display: 'flex', alignItems: 'flex-start', gap: 10, flex: 1, cursor: 'pointer' }} onClick={onToggle}>
          <div
            style={{
              width: 18,
              height: 18,
              borderRadius: 4,
              border: `1.5px solid ${checked ? 'var(--accent-blue)' : 'var(--border-medium)'}`,
              backgroundColor: checked ? 'var(--accent-blue)' : 'transparent',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              marginTop: 2,
              flexShrink: 0,
            }}
          >
            {checked && <Check size={12} color="#fff" strokeWidth={3} />}
          </div>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <span className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
                {action.title}
              </span>
              <span
                className={`badge ${action.risk === 'SAFE' ? 'badge-green' : 'badge-red'}`}
                style={{ fontSize: '9px', padding: '1px 5px' }}
              >
                {action.risk}
              </span>
              {action.isReversible && (
                <span className="badge badge-blue" style={{ fontSize: '9px', padding: '1px 5px' }}>
                  REVERSIBLE
                </span>
              )}
            </div>
            <div className="text-xs text-secondary" style={{ marginTop: 2 }}>
              {action.description}
            </div>
          </div>
        </div>

        <button
          className="btn btn-secondary"
          style={{ padding: '2px 8px', fontSize: '10px' }}
          onClick={(e) => {
            e.stopPropagation();
            setExpanded(!expanded);
          }}
        >
          {expanded ? 'Hide Details' : 'View Details'}
        </button>
      </div>

      {expanded && (
        <div
          style={{
            marginTop: 10,
            paddingTop: 10,
            borderTop: '1px solid var(--border-subtle)',
            fontSize: '11.5px',
            display: 'grid',
            gridTemplateColumns: 'repeat(2, 1fr)',
            gap: 8,
          }}
        >
          <div>
            <span className="text-muted">Why Recommended: </span>
            <span className="text-secondary">{action.rationale}</span>
          </div>
          <div>
            <span className="text-muted">Expected Benefit: </span>
            <span className="text-green">{action.expectedBenefit}</span>
          </div>
          <div>
            <span className="text-muted">Potential Consequence: </span>
            <span className="text-amber">{action.potentialConsequence}</span>
          </div>
          <div>
            <span className="text-muted">Transition: </span>
            <span className="mono text-secondary">
              {action.currentState} &rarr; {action.proposedState}
            </span>
          </div>
          <div>
            <span className="text-muted">Duration: </span>
            <span className="mono text-secondary">
              {action.durationMinutes === 0 ? 'Until restored manually' : `${action.durationMinutes} mins`}
            </span>
          </div>
          <div>
            <span className="text-muted">Target: </span>
            <span className="mono text-cyan">{action.targetIdentifier}</span>
          </div>
        </div>
      )}
    </div>
  );
};
