import React from 'react';
import { X, Sparkles, ArrowRight, Check } from 'lucide-react';
import { AutoAssignProposal } from '../types';

interface AutoAssignDiffModalProps {
  diff: AutoAssignProposal[];
  onClose: () => void;
  onApplyAll: () => void;
}

export const AutoAssignDiffModal: React.FC<AutoAssignDiffModalProps> = ({
  diff,
  onClose,
  onApplyAll,
}) => {
  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.6)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
      backdropFilter: 'blur(2px)'
    }}>
      <div style={{
        width: '580px',
        maxHeight: '80vh',
        backgroundColor: 'var(--bg-card)',
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-md)',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden'
      }}>
        {/* Header */}
        <div style={{
          padding: '16px 20px',
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Sparkles size={18} color="var(--accent)" />
            <div>
              <h3 style={{ fontSize: '15px', fontWeight: 700 }}>Auto-Assign Recommended Versions</h3>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
                {diff.length} {diff.length === 1 ? 'change' : 'changes'} suggested based on framework compatibility
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
          >
            <X size={18} />
          </button>
        </div>

        {/* Diff List */}
        <div style={{ padding: '16px 20px', overflowY: 'auto', flex: 1, display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {diff.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '30px', color: 'var(--text-muted)' }}>
              All projects are already running compatible Node.js versions. No changes suggested.
            </div>
          ) : (
            diff.map(item => (
              <div
                key={item.projectId}
                style={{
                  padding: '12px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: 'var(--bg-sidebar)',
                  border: '1px solid var(--border-color)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between'
                }}
              >
                <div>
                  <h4 style={{ fontSize: '13px', fontWeight: 600 }}>{item.projectName}</h4>
                  <p style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
                    {item.reason}
                  </p>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexShrink: 0 }}>
                  <span style={{
                    fontSize: '12px',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    backgroundColor: 'var(--border-color)',
                    color: item.currentVersion ? 'var(--text-secondary)' : 'var(--text-muted)'
                  }}>
                    {item.currentVersion ? `v${item.currentVersion}` : 'Unassigned'}
                  </span>
                  <ArrowRight size={14} color="var(--text-muted)" />
                  <span style={{
                    fontSize: '12px',
                    fontWeight: 600,
                    padding: '2px 8px',
                    borderRadius: '4px',
                    backgroundColor: 'var(--success-subtle)',
                    color: 'var(--success)',
                    border: '1px solid var(--success)'
                  }}>
                    v{item.proposedVersion}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Actions Footer */}
        <div style={{
          padding: '14px 20px',
          borderTop: '1px solid var(--border-color)',
          display: 'flex',
          justifyContent: 'flex-end',
          gap: '8px'
        }}>
          <button
            onClick={onClose}
            style={{
              padding: '8px 14px',
              borderRadius: 'var(--radius-md)',
              backgroundColor: 'transparent',
              border: '1px solid var(--border-color)',
              color: 'var(--text-secondary)',
              cursor: 'pointer'
            }}
          >
            Cancel
          </button>
          {diff.length > 0 && (
            <button
              onClick={onApplyAll}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '8px 16px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--accent)',
                color: '#fff',
                border: 'none',
                fontWeight: 600,
                cursor: 'pointer'
              }}
            >
              <Check size={16} />
              <span>Apply All ({diff.length})</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
