import React, { useState } from 'react';
import { X, Check } from 'lucide-react';
import { InstalledVersion } from '../types';

interface BulkAssignModalProps {
  selectedPaths: string[];
  installedVersions: InstalledVersion[];
  onClose: () => void;
  onApplyBulkAssign: (paths: string[], version: string) => void;
}

export const BulkAssignModal: React.FC<BulkAssignModalProps> = ({
  selectedPaths,
  installedVersions,
  onClose,
  onApplyBulkAssign,
}) => {
  const [selectedVersion, setSelectedVersion] = useState(
    installedVersions[0]?.version || '22.18.0'
  );

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
        width: '460px',
        backgroundColor: 'var(--bg-card)',
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-md)',
        overflow: 'hidden'
      }}>
        <div style={{
          padding: '16px 20px',
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}>
          <h3 style={{ fontSize: '15px', fontWeight: 700 }}>
            Assign Node.js to {selectedPaths.length} Projects
          </h3>
          <button
            onClick={onClose}
            style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
          >
            <X size={18} />
          </button>
        </div>

        <div style={{ padding: '20px', display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <p style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
            This will configure <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{selectedPaths.length}</span> selected projects to execute with the chosen Node runtime. Each project will have its local configuration updated safely.
          </p>

          <div>
            <label style={{ fontSize: '12px', fontWeight: 600, marginBottom: '6px', display: 'block' }}>
              Select Node.js Version:
            </label>
            <select
              value={selectedVersion}
              onChange={e => setSelectedVersion(e.target.value)}
              style={{
                width: '100%',
                padding: '9px 12px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--bg-sidebar)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                cursor: 'pointer',
                fontWeight: 500
              }}
            >
              <optgroup label="Installed Runtimes">
                {installedVersions.map(iv => (
                  <option key={iv.version} value={iv.version}>
                    Node.js v{iv.version} (installed)
                  </option>
                ))}
              </optgroup>
              <optgroup label="Available LTS Releases">
                <option value="22.18.0">Node.js 22 LTS (22.18.0)</option>
                <option value="20.19.5">Node.js 20 LTS (20.19.5)</option>
                <option value="18.20.8">Node.js 18 LTS (18.20.8)</option>
                <option value="16.20.2">Node.js 16 LTS (16.20.2)</option>
              </optgroup>
            </select>
          </div>

          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px', marginTop: '10px' }}>
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
            <button
              onClick={() => onApplyBulkAssign(selectedPaths, selectedVersion)}
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
              <span>Apply to {selectedPaths.length} Projects</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
