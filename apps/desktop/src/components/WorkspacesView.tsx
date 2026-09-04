import React, { useState } from 'react';
import { HardDrive, Plus, Trash2, RefreshCw, FolderGit2, Calendar } from 'lucide-react';
import { ProjectRecord } from '../types';

interface WorkspacesViewProps {
  workspaces: string[];
  projects: ProjectRecord[];
  lastScanTime?: string;
  isScanning: boolean;
  onAddWorkspace: (path: string) => void;
  onRemoveWorkspace: (path: string) => void;
  onScanNow: () => void;
}

export const WorkspacesView: React.FC<WorkspacesViewProps> = ({
  workspaces,
  projects,
  lastScanTime,
  isScanning,
  onAddWorkspace,
  onRemoveWorkspace,
  onScanNow,
}) => {
  const [newPath, setNewPath] = useState('');

  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault();
    if (newPath.trim()) {
      onAddWorkspace(newPath.trim());
      setNewPath('');
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '24px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <div>
          <h2 style={{ fontSize: '18px', fontWeight: 700 }}>Development Workspaces</h2>
          <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
            Root directories where your repositories are stored. NodePilot scans them recursively.
          </p>
        </div>

        <button
          onClick={onScanNow}
          disabled={isScanning}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            padding: '7px 14px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--accent)',
            color: '#fff',
            border: 'none',
            fontWeight: 600,
            cursor: isScanning ? 'not-allowed' : 'pointer',
            fontSize: '12px'
          }}
        >
          <RefreshCw size={14} className={isScanning ? 'animate-spin' : ''} />
          <span>{isScanning ? 'Scanning...' : 'Scan All Now'}</span>
        </button>
      </div>

      {/* Add Workspace Form */}
      <form
        onSubmit={handleAdd}
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
          placeholder="Enter directory path (e.g. C:\dev or /Users/username/dev)..."
          value={newPath}
          onChange={e => setNewPath(e.target.value)}
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
          disabled={!newPath.trim()}
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
            cursor: !newPath.trim() ? 'not-allowed' : 'pointer'
          }}
        >
          <Plus size={16} />
          <span>Add Folder</span>
        </button>
      </form>

      {/* Workspaces List */}
      <div>
        <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px' }}>
          Configured Workspaces ({workspaces.length})
        </h3>

        {workspaces.length === 0 ? (
          <div style={{
            padding: '36px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--bg-card)',
            border: '1px dashed var(--border-color)',
            textAlign: 'center',
            color: 'var(--text-muted)'
          }}>
            No workspaces added yet. Add a development directory above to get started.
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
            {workspaces.map(ws => {
              const projectCount = projects.filter(p => p.workspaceRoot === ws).length;
              return (
                <div
                  key={ws}
                  style={{
                    padding: '16px',
                    borderRadius: 'var(--radius-md)',
                    backgroundColor: 'var(--bg-card)',
                    border: '1px solid var(--border-color)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between'
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
                    <div style={{
                      width: '40px',
                      height: '40px',
                      borderRadius: '8px',
                      backgroundColor: 'var(--bg-sidebar)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center'
                    }}>
                      <HardDrive size={20} color="var(--accent)" />
                    </div>
                    <div>
                      <h4 style={{ fontSize: '14px', fontWeight: 600, fontFamily: 'var(--font-mono)' }}>{ws}</h4>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginTop: '4px', fontSize: '11px', color: 'var(--text-muted)' }}>
                        <span style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                          <FolderGit2 size={12} />
                          <span>{projectCount} {projectCount === 1 ? 'project' : 'projects'} found</span>
                        </span>
                        {lastScanTime && (
                          <span style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                            <Calendar size={12} />
                            <span>Last scanned: {new Date(lastScanTime).toLocaleTimeString()}</span>
                          </span>
                        )}
                      </div>
                    </div>
                  </div>

                  <button
                    onClick={() => onRemoveWorkspace(ws)}
                    title="Remove workspace"
                    style={{
                      padding: '7px',
                      borderRadius: 'var(--radius-sm)',
                      backgroundColor: 'transparent',
                      color: 'var(--text-muted)',
                      border: 'none',
                      cursor: 'pointer'
                    }}
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};
