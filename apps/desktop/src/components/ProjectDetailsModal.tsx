import React from 'react';
import { 
  X, 
  Terminal, 
  Code2, 
  FolderOpen, 
  Trash2, 
  CheckCircle2, 
  AlertTriangle
} from 'lucide-react';
import { ProjectRecord, InstalledVersion } from '../types';

interface ProjectDetailsModalProps {
  project: ProjectRecord | null;
  installedVersions: InstalledVersion[];
  onClose: () => void;
  onAssignNode: (projectPath: string, version: string) => void;
  onOpenTerminal: (projectPath: string) => void;
  onOpenEditor: (projectPath: string) => void;
  onOpenFolder: (projectPath: string) => void;
  onRemoveAssignment: (projectPath: string) => void;
}

export const ProjectDetailsModal: React.FC<ProjectDetailsModalProps> = ({
  project,
  installedVersions,
  onClose,
  onAssignNode,
  onOpenTerminal,
  onOpenEditor,
  onOpenFolder,
  onRemoveAssignment,
}) => {
  if (!project) return null;

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
        width: '560px',
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
          <div>
            <h3 style={{ fontSize: '16px', fontWeight: 700 }}>{project.name}</h3>
            <p style={{ fontSize: '11px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
              {project.path}
            </p>
          </div>
          <button
            onClick={onClose}
            style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
          >
            <X size={18} />
          </button>
        </div>

        {/* Content Body */}
        <div style={{ padding: '20px', display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Metadata Grid */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
            <div style={{ padding: '10px', backgroundColor: 'var(--bg-sidebar)', borderRadius: 'var(--radius-sm)' }}>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Framework</span>
              <p style={{ fontWeight: 600, fontSize: '13px', marginTop: '2px' }}>
                {project.framework} {project.frameworkVersion ? `v${project.frameworkVersion}` : ''}
              </p>
            </div>

            <div style={{ padding: '10px', backgroundColor: 'var(--bg-sidebar)', borderRadius: 'var(--radius-sm)' }}>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Package Manager</span>
              <p style={{ fontWeight: 600, fontSize: '13px', marginTop: '2px', textTransform: 'uppercase' }}>
                {project.packageManager}
              </p>
            </div>

            <div style={{ padding: '10px', backgroundColor: 'var(--bg-sidebar)', borderRadius: 'var(--radius-sm)' }}>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Assigned Node.js</span>
              <p style={{ fontWeight: 600, fontSize: '13px', marginTop: '2px', color: 'var(--accent)' }}>
                {project.assignedNode ? `v${project.assignedNode}` : 'None assigned'}
              </p>
            </div>

            <div style={{ padding: '10px', backgroundColor: 'var(--bg-sidebar)', borderRadius: 'var(--radius-sm)' }}>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Config Source</span>
              <p style={{ fontWeight: 500, fontSize: '12px', marginTop: '2px' }}>
                {project.isInherited ? 'Parent inheritance' : project.configSource}
              </p>
            </div>
          </div>

          {/* Compatibility Banner */}
          <div style={{
            padding: '12px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: project.compatibilityStatus === 'compatible' ? 'var(--success-subtle)' : 'var(--danger-subtle)',
            border: `1px solid ${project.compatibilityStatus === 'compatible' ? 'var(--success)' : 'var(--danger)'}`,
            display: 'flex',
            alignItems: 'flex-start',
            gap: '10px'
          }}>
            {project.compatibilityStatus === 'compatible' ? (
              <CheckCircle2 size={18} color="var(--success)" style={{ flexShrink: 0, marginTop: '2px' }} />
            ) : (
              <AlertTriangle size={18} color="var(--danger)" style={{ flexShrink: 0, marginTop: '2px' }} />
            )}
            <div>
              <span style={{ fontWeight: 600, fontSize: '12px' }}>
                {project.compatibilityStatus === 'compatible' ? 'Environment Compatible' : 'Compatibility Warning'}
              </span>
              <p style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '2px' }}>
                {project.compatibilityMessage || 'Node version satisfies requirements.'}
              </p>
            </div>
          </div>

          {/* Quick Version Switcher */}
          <div>
            <label style={{ fontSize: '12px', fontWeight: 600, marginBottom: '6px', display: 'block' }}>
              Assign Different Node.js Version:
            </label>
            <select
              value={project.assignedNode || ''}
              onChange={e => onAssignNode(project.path, e.target.value)}
              style={{
                width: '100%',
                padding: '8px 10px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--bg-sidebar)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                cursor: 'pointer'
              }}
            >
              <option value="" disabled>Choose version...</option>
              <optgroup label="Installed Runtimes">
                {installedVersions.map(iv => (
                  <option key={iv.version} value={iv.version}>
                    v{iv.version}
                  </option>
                ))}
              </optgroup>
              <optgroup label="Official Releases">
                <option value="22.18.0">v22.18.0 (LTS Iron)</option>
                <option value="20.19.5">v20.19.5 (LTS Hydrogen)</option>
                <option value="18.20.8">v18.20.8 (LTS Gallium)</option>
                <option value="16.20.2">v16.20.2 (LTS Fermium)</option>
              </optgroup>
            </select>
          </div>

          {/* Action Buttons */}
          <div style={{ display: 'flex', gap: '8px', marginTop: '8px' }}>
            <button
              onClick={() => onOpenTerminal(project.path)}
              style={{
                flex: 1,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '6px',
                padding: '9px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--accent)',
                color: '#fff',
                border: 'none',
                fontWeight: 600,
                cursor: 'pointer'
              }}
            >
              <Terminal size={15} />
              <span>Open Terminal</span>
            </button>

            <button
              onClick={() => onOpenEditor(project.path)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '9px 14px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--border-color)',
                color: 'var(--text-primary)',
                border: 'none',
                cursor: 'pointer',
                fontWeight: 500
              }}
            >
              <Code2 size={15} />
              <span>VS Code</span>
            </button>

            <button
              onClick={() => onOpenFolder(project.path)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '9px 14px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--border-color)',
                color: 'var(--text-primary)',
                border: 'none',
                cursor: 'pointer',
                fontWeight: 500
              }}
            >
              <FolderOpen size={15} />
              <span>Folder</span>
            </button>

            {project.assignedNode && (
              <button
                onClick={() => onRemoveAssignment(project.path)}
                title="Remove .nodepilot.local override"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  padding: '9px 12px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: 'var(--danger-subtle)',
                  color: 'var(--danger)',
                  border: 'none',
                  cursor: 'pointer'
                }}
              >
                <Trash2 size={15} />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
