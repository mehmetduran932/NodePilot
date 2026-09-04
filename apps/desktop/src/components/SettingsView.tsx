import React from 'react';
import { 
  Moon, 
  Sun, 
  Monitor, 
  Shield, 
  RefreshCw
} from 'lucide-react';
import { Settings, UpdateCheckResult } from '../types';

interface SettingsViewProps {
  settings: Settings;
  onUpdateSettings: (newSettings: Settings) => void;
  updateInfo: UpdateCheckResult | null;
  isCheckingUpdate: boolean;
  onCheckAppUpdate: () => void;
}

export const SettingsView: React.FC<SettingsViewProps> = ({
  settings,
  onUpdateSettings,
  updateInfo,
  isCheckingUpdate,
  onCheckAppUpdate,
}) => {
  const handleChange = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    onUpdateSettings({
      ...settings,
      [key]: value,
    });
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto', padding: '24px' }}>
      <div style={{ marginBottom: '20px' }}>
        <h2 style={{ fontSize: '18px', fontWeight: 700 }}>Settings</h2>
        <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
          Configure scanning behavior, terminal preferences, appearance, and privacy.
        </p>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', maxWidth: '640px' }}>
        {/* Appearance / Theme */}
        <div style={{
          padding: '18px',
          borderRadius: 'var(--radius-md)',
          backgroundColor: 'var(--bg-card)',
          border: '1px solid var(--border-color)'
        }}>
          <h4 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px' }}>Appearance</h4>
          <div style={{ display: 'flex', gap: '10px' }}>
            {(['system', 'dark', 'light'] as const).map(theme => (
              <button
                key={theme}
                onClick={() => handleChange('theme', theme)}
                style={{
                  flex: 1,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '8px',
                  padding: '9px 12px',
                  borderRadius: 'var(--radius-sm)',
                  backgroundColor: settings.theme === theme ? 'var(--accent-subtle)' : 'var(--bg-sidebar)',
                  border: `1px solid ${settings.theme === theme ? 'var(--accent)' : 'var(--border-color)'}`,
                  color: settings.theme === theme ? 'var(--accent)' : 'var(--text-primary)',
                  fontWeight: settings.theme === theme ? 600 : 500,
                  cursor: 'pointer',
                  textTransform: 'capitalize'
                }}
              >
                {theme === 'system' && <Monitor size={15} />}
                {theme === 'dark' && <Moon size={15} />}
                {theme === 'light' && <Sun size={15} />}
                <span>{theme}</span>
              </button>
            ))}
          </div>
        </div>

        {/* General Preferences */}
        <div style={{
          padding: '18px',
          borderRadius: 'var(--radius-md)',
          backgroundColor: 'var(--bg-card)',
          border: '1px solid var(--border-color)',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px'
        }}>
          <h4 style={{ fontSize: '14px', fontWeight: 600 }}>Preferences</h4>

          <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
            <div>
              <span style={{ fontWeight: 500, fontSize: '13px' }}>Scan workspaces on launch</span>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Automatically refresh projects when opening NodePilot</p>
            </div>
            <input
              type="checkbox"
              checked={settings.scanOnStartup}
              onChange={e => handleChange('scanOnStartup', e.target.checked)}
              style={{ cursor: 'pointer' }}
            />
          </label>

          <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
            <div>
              <span style={{ fontWeight: 500, fontSize: '13px' }}>Auto-check for Node.js updates</span>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Notify when new LTS versions are released</p>
            </div>
            <input
              type="checkbox"
              checked={settings.autoCheckNodeUpdates}
              onChange={e => handleChange('autoCheckNodeUpdates', e.target.checked)}
              style={{ cursor: 'pointer' }}
            />
          </label>

          <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
            <div>
              <span style={{ fontWeight: 500, fontSize: '13px' }}>Preferred Terminal</span>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Terminal to open when launching projects</p>
            </div>
            <select
              value={settings.preferredTerminal}
              onChange={e => handleChange('preferredTerminal', e.target.value)}
              style={{
                padding: '6px 10px',
                borderRadius: 'var(--radius-sm)',
                backgroundColor: 'var(--bg-sidebar)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                fontSize: '12px'
              }}
            >
              <option value="system">System Default</option>
              <option value="wt">Windows Terminal</option>
              <option value="powershell">PowerShell</option>
              <option value="cmd">Command Prompt</option>
              <option value="iterm">iTerm2 (macOS)</option>
            </select>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
            <div>
              <span style={{ fontWeight: 500, fontSize: '13px' }}>Preferred Code Editor</span>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Editor executable for opening projects</p>
            </div>
            <select
              value={settings.preferredEditor}
              onChange={e => handleChange('preferredEditor', e.target.value)}
              style={{
                padding: '6px 10px',
                borderRadius: 'var(--radius-sm)',
                backgroundColor: 'var(--bg-sidebar)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                fontSize: '12px'
              }}
            >
              <option value="code">Visual Studio Code (code)</option>
              <option value="webstorm">WebStorm</option>
              <option value="cursor">Cursor</option>
            </select>
          </label>
        </div>

        {/* Privacy & Telemetry */}
        <div style={{
          padding: '18px',
          borderRadius: 'var(--radius-md)',
          backgroundColor: 'var(--bg-card)',
          border: '1px solid var(--border-color)'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
            <Shield size={16} color="var(--success)" />
            <h4 style={{ fontSize: '14px', fontWeight: 600 }}>Privacy & Data Security</h4>
          </div>
          <p style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '12px' }}>
            NodePilot adheres to strict open-source privacy standards. Telemetry is permanently OFF by default.
            No personal project information or directory structures are ever transmitted.
          </p>

          <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
            <div>
              <span style={{ fontWeight: 500, fontSize: '13px' }}>Anonymous Telemetry</span>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Always disabled by default</p>
            </div>
            <input
              type="checkbox"
              checked={settings.telemetry}
              onChange={e => handleChange('telemetry', e.target.checked)}
              disabled={true} // Strictly disabled
              style={{ cursor: 'not-allowed' }}
            />
          </label>
        </div>

        {/* Updates & Version Info */}
        <div style={{
          padding: '18px',
          borderRadius: 'var(--radius-md)',
          backgroundColor: 'var(--bg-card)',
          border: '1px solid var(--border-color)'
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <h4 style={{ fontSize: '14px', fontWeight: 600 }}>NodePilot Version</h4>
              <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>Current version: v0.1.0</p>
            </div>

            <button
              onClick={onCheckAppUpdate}
              disabled={isCheckingUpdate}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '7px 12px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--bg-sidebar)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                fontSize: '12px',
                cursor: isCheckingUpdate ? 'not-allowed' : 'pointer'
              }}
            >
              <RefreshCw size={13} className={isCheckingUpdate ? 'animate-spin' : ''} />
              <span>{isCheckingUpdate ? 'Checking...' : 'Check for Updates'}</span>
            </button>
          </div>

          {updateInfo && (
            <div style={{
              marginTop: '12px',
              padding: '10px 14px',
              borderRadius: 'var(--radius-sm)',
              backgroundColor: updateInfo.updateAvailable ? 'var(--accent-subtle)' : 'var(--bg-sidebar)',
              border: `1px solid ${updateInfo.updateAvailable ? 'var(--accent)' : 'var(--border-color)'}`,
              fontSize: '12px'
            }}>
              {updateInfo.updateAvailable ? (
                <div>
                  <span style={{ fontWeight: 600, color: 'var(--accent)' }}>
                    NodePilot {updateInfo.latestVersion} is available!
                  </span>
                  <div style={{ marginTop: '4px', color: 'var(--text-muted)' }}>
                    Upgrade command: <code>{updateInfo.packageManagerCommand}</code>
                  </div>
                </div>
              ) : (
                <span style={{ color: 'var(--success)' }}>NodePilot is up to date.</span>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
