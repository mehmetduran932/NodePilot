import React, { useState } from 'react';
import { 
  Layers, 
  Download, 
  Trash2, 
  AlertTriangle, 
  CheckCircle2, 
  RefreshCw,
  FolderGit2
} from 'lucide-react';
import { InstalledVersion, NodeRelease } from '../types';

interface NodeManagerViewProps {
  installedVersions: InstalledVersion[];
  availableReleases: NodeRelease[];
  isInstalling: boolean;
  installingVersion: string | null;
  onInstallVersion: (version: string) => void;
  onUninstallVersion: (version: string) => void;
  onRefreshReleases: () => void;
}

export const NodeManagerView: React.FC<NodeManagerViewProps> = ({
  installedVersions,
  availableReleases,
  isInstalling,
  installingVersion,
  onInstallVersion,
  onUninstallVersion,
  onRefreshReleases,
}) => {
  const [customVersion, setCustomVersion] = useState('');
  const [confirmDeleteVersion, setConfirmDeleteVersion] = useState<InstalledVersion | null>(null);

  const totalDiskBytes = installedVersions.reduce((sum, v) => sum + v.sizeBytes, 0);
  const totalDiskMb = (totalDiskBytes / 1_048_576).toFixed(1);

  // Filter available LTS releases that are not yet installed
  const ltsReleases = availableReleases.filter(r => r.lts !== false);

  const handleCustomInstall = (e: React.FormEvent) => {
    e.preventDefault();
    if (customVersion.trim()) {
      onInstallVersion(customVersion.trim());
      setCustomVersion('');
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '24px' }}>
      {/* Top Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <div>
          <h2 style={{ fontSize: '18px', fontWeight: 700 }}>Node.js Runtimes</h2>
          <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
            Isolated Node installations managed exclusively by NodePilot. Total disk usage: {totalDiskMb} MB.
          </p>
        </div>

        <button
          onClick={onRefreshReleases}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            padding: '7px 12px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--bg-card)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-secondary)',
            cursor: 'pointer',
            fontSize: '12px'
          }}
        >
          <RefreshCw size={14} />
          <span>Refresh Releases</span>
        </button>
      </div>

      {/* Manual Install Form */}
      <form
        onSubmit={handleCustomInstall}
        style={{
          display: 'flex',
          gap: '10px',
          padding: '14px 18px',
          borderRadius: 'var(--radius-md)',
          backgroundColor: 'var(--bg-card)',
          border: '1px solid var(--border-color)',
          marginBottom: '24px'
        }}
      >
        <input
          type="text"
          placeholder="Install specific version or alias (e.g. 22, 22.18.0, lts, latest)..."
          value={customVersion}
          onChange={e => setCustomVersion(e.target.value)}
          disabled={isInstalling}
          style={{
            flex: 1,
            padding: '8px 12px',
            borderRadius: 'var(--radius-sm)',
            backgroundColor: 'var(--bg-sidebar)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-primary)',
            fontSize: '13px'
          }}
        />
        <button
          type="submit"
          disabled={isInstalling || !customVersion.trim()}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            padding: '8px 16px',
            borderRadius: 'var(--radius-sm)',
            backgroundColor: 'var(--accent)',
            color: '#fff',
            border: 'none',
            fontWeight: 600,
            cursor: isInstalling || !customVersion.trim() ? 'not-allowed' : 'pointer'
          }}
        >
          <Download size={15} />
          <span>{isInstalling ? 'Installing...' : 'Install'}</span>
        </button>
      </form>

      {/* Installed Runtimes Section */}
      <div style={{ marginBottom: '32px' }}>
        <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px', display: 'flex', alignItems: 'center', gap: '8px' }}>
          <CheckCircle2 size={16} color="var(--success)" />
          <span>Installed Runtimes ({installedVersions.length})</span>
        </h3>

        {installedVersions.length === 0 ? (
          <div style={{
            padding: '30px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--bg-card)',
            border: '1px dashed var(--border-color)',
            textAlign: 'center',
            color: 'var(--text-muted)'
          }}>
            No Node.js versions installed yet. Click an LTS version below to install.
          </div>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '12px' }}>
            {installedVersions.map(iv => {
              const mb = (iv.sizeBytes / 1_048_576).toFixed(1);
              return (
                <div
                  key={iv.version}
                  style={{
                    padding: '16px',
                    borderRadius: 'var(--radius-md)',
                    backgroundColor: 'var(--bg-card)',
                    border: '1px solid var(--border-color)',
                    display: 'flex',
                    flexDirection: 'column',
                    justifyContent: 'space-between',
                    gap: '12px'
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                    <div>
                      <h4 style={{ fontSize: '15px', fontWeight: 700 }}>Node.js v{iv.version}</h4>
                      <p style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
                        {mb} MB on disk
                      </p>
                    </div>

                    <button
                      onClick={() => setConfirmDeleteVersion(iv)}
                      title="Uninstall runtime"
                      style={{
                        padding: '6px',
                        borderRadius: 'var(--radius-sm)',
                        backgroundColor: 'var(--danger-subtle)',
                        color: 'var(--danger)',
                        border: 'none',
                        cursor: 'pointer'
                      }}
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>

                  <div style={{
                    padding: '8px 10px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: 'var(--bg-sidebar)',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    fontSize: '11px',
                    color: iv.projectsUsingCount > 0 ? 'var(--text-primary)' : 'var(--text-muted)'
                  }}>
                    <FolderGit2 size={13} color="var(--text-muted)" />
                    <span>
                      {iv.projectsUsingCount > 0 
                        ? `Used by ${iv.projectsUsingCount} project${iv.projectsUsingCount === 1 ? '' : 's'}`
                        : 'Not actively assigned to any project'}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Available LTS Versions */}
      <div>
        <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px', display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Layers size={16} color="var(--accent)" />
          <span>Available Long-Term Support (LTS) Releases</span>
        </h3>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '12px' }}>
          {ltsReleases.slice(0, 6).map(rel => {
            const clean = rel.version.replace(/^v/, '');
            const isInstalled = installedVersions.some(iv => iv.version === clean);
            const isCurrentInstalling = isInstalling && installingVersion === clean;
            const ltsCodename = typeof rel.lts === 'string' ? rel.lts : undefined;

            return (
              <div
                key={rel.version}
                style={{
                  padding: '14px 16px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: 'var(--bg-card)',
                  border: '1px solid var(--border-color)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between'
                }}
              >
                <div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ fontWeight: 600, fontSize: '13px' }}>Node.js {rel.version}</span>
                    {ltsCodename && (
                      <span style={{
                        fontSize: '10px',
                        padding: '1px 6px',
                        borderRadius: '4px',
                        backgroundColor: 'var(--accent-subtle)',
                        color: 'var(--accent)',
                        fontWeight: 600
                      }}>
                        LTS {ltsCodename}
                      </span>
                    )}
                  </div>
                  <p style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
                    Released: {rel.date}
                  </p>
                </div>

                {isInstalled ? (
                  <span style={{
                    fontSize: '11px',
                    fontWeight: 600,
                    color: 'var(--success)',
                    padding: '4px 8px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: 'var(--success-subtle)'
                  }}>
                    Installed
                  </span>
                ) : (
                  <button
                    onClick={() => onInstallVersion(clean)}
                    disabled={isInstalling}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '5px',
                      padding: '6px 12px',
                      borderRadius: 'var(--radius-sm)',
                      backgroundColor: 'var(--accent)',
                      color: '#fff',
                      border: 'none',
                      fontSize: '12px',
                      fontWeight: 600,
                      cursor: isInstalling ? 'not-allowed' : 'pointer'
                    }}
                  >
                    <Download size={13} />
                    <span>{isCurrentInstalling ? 'Installing...' : 'Install'}</span>
                  </button>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Confirmation Modal when uninstalling a version used by projects */}
      {confirmDeleteVersion && (
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
          zIndex: 1100,
          backdropFilter: 'blur(2px)'
        }}>
          <div style={{
            width: '440px',
            backgroundColor: 'var(--bg-card)',
            borderRadius: 'var(--radius-lg)',
            border: '1px solid var(--border-color)',
            boxShadow: 'var(--shadow-md)',
            padding: '20px'
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--danger)', marginBottom: '12px' }}>
              <AlertTriangle size={22} />
              <h3 style={{ fontSize: '15px', fontWeight: 700 }}>Uninstall Node v{confirmDeleteVersion.version}?</h3>
            </div>

            {confirmDeleteVersion.projectsUsingCount > 0 ? (
              <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '16px' }}>
                <span style={{ color: 'var(--danger)', fontWeight: 600 }}>Warning:</span> Node {confirmDeleteVersion.version} is currently assigned to{' '}
                <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{confirmDeleteVersion.projectsUsingCount} active projects</span> ({confirmDeleteVersion.projectsUsing.slice(0, 3).join(', ')}{confirmDeleteVersion.projectsUsingCount > 3 ? '...' : ''}).
                <br /><br />
                Uninstalling it will cause those projects to require re-installing Node runtime when executing scripts.
              </p>
            ) : (
              <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '16px' }}>
                Are you sure you want to remove Node.js v{confirmDeleteVersion.version}? This will free {(confirmDeleteVersion.sizeBytes / 1_048_576).toFixed(1)} MB of disk space.
              </p>
            )}

            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
              <button
                onClick={() => setConfirmDeleteVersion(null)}
                style={{
                  padding: '7px 14px',
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
                onClick={() => {
                  onUninstallVersion(confirmDeleteVersion.version);
                  setConfirmDeleteVersion(null);
                }}
                style={{
                  padding: '7px 16px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: 'var(--danger)',
                  color: '#fff',
                  border: 'none',
                  fontWeight: 600,
                  cursor: 'pointer'
                }}
              >
                Uninstall Runtime
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
