import React from 'react';
import { 
  ShieldCheck, 
  CheckCircle2
} from 'lucide-react';
import { EnvironmentReport, IntegrationStatus } from '../types';

interface DoctorViewProps {
  report: EnvironmentReport | null;
  integration: IntegrationStatus | null;
  isTogglingIntegration: boolean;
  onToggleIntegration: (enable: boolean) => void;
  onRefreshDoctor: () => void;
}

export const DoctorView: React.FC<DoctorViewProps> = ({
  report,
  integration,
  isTogglingIntegration,
  onToggleIntegration,
  onRefreshDoctor,
}) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '24px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <div>
          <h2 style={{ fontSize: '18px', fontWeight: 700 }}>NodePilot Doctor</h2>
          <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
            System environment diagnostics, manager coexistence status, and shell integration health.
          </p>
        </div>

        <button
          onClick={onRefreshDoctor}
          style={{
            padding: '7px 14px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--bg-card)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-secondary)',
            cursor: 'pointer',
            fontSize: '12px'
          }}
        >
          Re-check Environment
        </button>
      </div>

      {/* Safe Coexistence Mode Banner */}
      <div style={{
        padding: '16px',
        borderRadius: 'var(--radius-md)',
        backgroundColor: 'var(--bg-card)',
        border: '1px solid var(--border-color)',
        marginBottom: '20px',
        display: 'flex',
        alignItems: 'flex-start',
        gap: '14px'
      }}>
        <div style={{
          width: '36px',
          height: '36px',
          borderRadius: '8px',
          backgroundColor: 'var(--success-subtle)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexShrink: 0
        }}>
          <ShieldCheck size={20} color="var(--success)" />
        </div>
        <div>
          <h3 style={{ fontSize: '14px', fontWeight: 700 }}>
            {integration?.isEnabled ? 'Direct Shell Integration Active' : 'Safe Coexistence Mode Active (Default)'}
          </h3>
          <p style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            NodePilot never deletes, modifies, or overwrites other Node managers (nvm, fnm, Volta, mise, asdf).
            All Node runtimes remain strictly isolated in NodePilot's directory.
          </p>
        </div>
      </div>

      {/* Shell Integration Card */}
      <div style={{
        padding: '18px',
        borderRadius: 'var(--radius-md)',
        backgroundColor: 'var(--bg-card)',
        border: '1px solid var(--border-color)',
        marginBottom: '20px'
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <h4 style={{ fontSize: '14px', fontWeight: 600 }}>Shell & PATH Integration</h4>
            <p style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '2px' }}>
              When enabled, adds NodePilot shims to your User PATH so standard terminals automatically run the project's assigned Node.
            </p>
          </div>

          <button
            onClick={() => onToggleIntegration(!integration?.isEnabled)}
            disabled={isTogglingIntegration}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '7px 14px',
              borderRadius: 'var(--radius-md)',
              backgroundColor: integration?.isEnabled ? 'var(--danger-subtle)' : 'var(--accent)',
              color: integration?.isEnabled ? 'var(--danger)' : '#fff',
              border: 'none',
              fontWeight: 600,
              cursor: isTogglingIntegration ? 'not-allowed' : 'pointer'
            }}
          >
            {integration?.isEnabled ? 'Disable Shell Integration' : 'Enable Shell Integration'}
          </button>
        </div>

        <div style={{
          marginTop: '14px',
          padding: '10px 14px',
          backgroundColor: 'var(--bg-sidebar)',
          borderRadius: 'var(--radius-sm)',
          fontSize: '12px',
          fontFamily: 'var(--font-mono)'
        }}>
          <div>Bin Path: {integration?.binDir}</div>
          <div style={{ marginTop: '4px', color: 'var(--text-muted)' }}>
            Rollback backup: {integration?.backupAvailable ? 'Available (restores prior PATH on disable)' : 'None needed'}
          </div>
        </div>
      </div>

      {/* Detected Managers */}
      <div style={{ marginBottom: '20px' }}>
        <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '10px' }}>
          Detected Node Managers & System Runtimes
        </h3>

        {!report || report.existingManagers.length === 0 ? (
          <div style={{
            padding: '16px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--bg-card)',
            border: '1px solid var(--border-color)',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            color: 'var(--text-secondary)'
          }}>
            <CheckCircle2 size={16} color="var(--success)" />
            <span>No third-party Node version managers detected in PATH.</span>
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {report.existingManagers.map((mgr, idx) => (
              <div
                key={idx}
                style={{
                  padding: '12px 16px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: 'var(--bg-card)',
                  border: '1px solid var(--border-color)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between'
                }}
              >
                <div>
                  <span style={{ fontWeight: 600, fontSize: '13px' }}>{mgr.name}</span>
                  <span style={{ fontSize: '12px', color: 'var(--text-muted)', marginLeft: '8px' }}>
                    {mgr.description}
                  </span>
                  {mgr.path && (
                    <div style={{ fontSize: '11px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)', marginTop: '2px' }}>
                      {mgr.path}
                    </div>
                  )}
                </div>

                <span style={{
                  fontSize: '11px',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  backgroundColor: 'var(--warning-subtle)',
                  color: 'var(--warning)',
                  fontWeight: 600
                }}>
                  Detected (Untouched)
                </span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* System Node Runtime */}
      <div>
        <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '10px' }}>
          System Default Node.js Binary
        </h3>
        <div style={{
          padding: '14px 16px',
          borderRadius: 'var(--radius-md)',
          backgroundColor: 'var(--bg-card)',
          border: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}>
          <div>
            <div style={{ fontWeight: 600, fontSize: '13px' }}>
              {report?.systemNodeVersion ? `Node.js ${report.systemNodeVersion}` : 'No system Node found in PATH'}
            </div>
            {report?.systemNodePath && (
              <div style={{ fontSize: '11px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)', marginTop: '2px' }}>
                {report.systemNodePath}
              </div>
            )}
          </div>
          <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
            Read-only check
          </span>
        </div>
      </div>
    </div>
  );
};
