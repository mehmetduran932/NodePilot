import React from 'react';
import { 
  FolderGit2, 
  Layers, 
  HardDrive, 
  Stethoscope, 
  Settings as SettingsIcon, 
  Compass,
  RefreshCw
} from 'lucide-react';

export type NavTab = 'projects' | 'node' | 'workspaces' | 'doctor' | 'settings';

interface NavigationProps {
  currentTab: NavTab;
  onSelectTab: (tab: NavTab) => void;
  projectCount: number;
  installedCount: number;
  onQuickScan: () => void;
  isScanning: boolean;
}

export const Navigation: React.FC<NavigationProps> = ({
  currentTab,
  onSelectTab,
  projectCount,
  installedCount,
  onQuickScan,
  isScanning,
}) => {
  return (
    <aside style={{
      width: '240px',
      backgroundColor: 'var(--bg-sidebar)',
      borderRight: '1px solid var(--border-color)',
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      flexShrink: 0
    }}>
      {/* Brand Header */}
      <div style={{
        padding: '20px 16px 16px 16px',
        display: 'flex',
        alignItems: 'center',
        gap: '10px',
        borderBottom: '1px solid var(--border-color)'
      }}>
        <div style={{
          width: '32px',
          height: '32px',
          borderRadius: '8px',
          backgroundColor: 'var(--accent)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#fff',
          boxShadow: 'var(--shadow-sm)'
        }}>
          <Compass size={20} />
        </div>
        <div>
          <h1 style={{ fontSize: '15px', fontWeight: 700, letterSpacing: '-0.2px' }}>NodePilot</h1>
          <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Project-aware Node</p>
        </div>
      </div>

      {/* Nav Menu */}
      <nav style={{ flex: 1, padding: '16px 10px', display: 'flex', flexDirection: 'column', gap: '4px' }}>
        <button
          onClick={() => onSelectTab('projects')}
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '9px 12px',
            borderRadius: 'var(--radius-md)',
            border: 'none',
            background: currentTab === 'projects' ? 'var(--accent-subtle)' : 'transparent',
            color: currentTab === 'projects' ? 'var(--accent)' : 'var(--text-secondary)',
            fontWeight: currentTab === 'projects' ? 600 : 500,
            cursor: 'pointer',
            textAlign: 'left'
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <FolderGit2 size={16} />
            <span>Projects</span>
          </div>
          {projectCount > 0 && (
            <span style={{
              fontSize: '11px',
              padding: '2px 7px',
              borderRadius: '10px',
              backgroundColor: currentTab === 'projects' ? 'var(--accent)' : 'var(--border-color)',
              color: currentTab === 'projects' ? '#fff' : 'var(--text-muted)'
            }}>
              {projectCount}
            </span>
          )}
        </button>

        <button
          onClick={() => onSelectTab('node')}
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '9px 12px',
            borderRadius: 'var(--radius-md)',
            border: 'none',
            background: currentTab === 'node' ? 'var(--accent-subtle)' : 'transparent',
            color: currentTab === 'node' ? 'var(--accent)' : 'var(--text-secondary)',
            fontWeight: currentTab === 'node' ? 600 : 500,
            cursor: 'pointer',
            textAlign: 'left'
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <Layers size={16} />
            <span>Node.js</span>
          </div>
          {installedCount > 0 && (
            <span style={{
              fontSize: '11px',
              padding: '2px 7px',
              borderRadius: '10px',
              backgroundColor: currentTab === 'node' ? 'var(--accent)' : 'var(--border-color)',
              color: currentTab === 'node' ? '#fff' : 'var(--text-muted)'
            }}>
              {installedCount}
            </span>
          )}
        </button>

        <button
          onClick={() => onSelectTab('workspaces')}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
            padding: '9px 12px',
            borderRadius: 'var(--radius-md)',
            border: 'none',
            background: currentTab === 'workspaces' ? 'var(--accent-subtle)' : 'transparent',
            color: currentTab === 'workspaces' ? 'var(--accent)' : 'var(--text-secondary)',
            fontWeight: currentTab === 'workspaces' ? 600 : 500,
            cursor: 'pointer',
            textAlign: 'left'
          }}
        >
          <HardDrive size={16} />
          <span>Workspaces</span>
        </button>

        <button
          onClick={() => onSelectTab('doctor')}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
            padding: '9px 12px',
            borderRadius: 'var(--radius-md)',
            border: 'none',
            background: currentTab === 'doctor' ? 'var(--accent-subtle)' : 'transparent',
            color: currentTab === 'doctor' ? 'var(--accent)' : 'var(--text-secondary)',
            fontWeight: currentTab === 'doctor' ? 600 : 500,
            cursor: 'pointer',
            textAlign: 'left'
          }}
        >
          <Stethoscope size={16} />
          <span>Doctor</span>
        </button>

        <button
          onClick={() => onSelectTab('settings')}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
            padding: '9px 12px',
            borderRadius: 'var(--radius-md)',
            border: 'none',
            background: currentTab === 'settings' ? 'var(--accent-subtle)' : 'transparent',
            color: currentTab === 'settings' ? 'var(--accent)' : 'var(--text-secondary)',
            fontWeight: currentTab === 'settings' ? 600 : 500,
            cursor: 'pointer',
            textAlign: 'left'
          }}
        >
          <SettingsIcon size={16} />
          <span>Settings</span>
        </button>
      </nav>

      {/* Quick Rescan CTA in Footer */}
      <div style={{ padding: '14px', borderTop: '1px solid var(--border-color)' }}>
        <button
          onClick={onQuickScan}
          disabled={isScanning}
          style={{
            width: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            padding: '8px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--bg-card)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-primary)',
            cursor: isScanning ? 'not-allowed' : 'pointer',
            fontWeight: 500,
            fontSize: '12px'
          }}
        >
          <RefreshCw size={14} className={isScanning ? 'animate-spin' : ''} />
          <span>{isScanning ? 'Scanning...' : 'Rescan Workspaces'}</span>
        </button>
      </div>
    </aside>
  );
};
