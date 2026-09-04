import React, { useState } from 'react';
import { 
  Compass, 
  HardDrive, 
  RefreshCw, 
  ShieldCheck, 
  CheckCircle2, 
  ArrowRight,
  FolderGit2
} from 'lucide-react';
import { EnvironmentReport } from '../types';

interface OnboardingModalProps {
  report: EnvironmentReport | null;
  onAddWorkspaceAndScan: (path: string) => Promise<number>;
  onFinishOnboarding: (enableIntegration: boolean) => void;
}

export const OnboardingModal: React.FC<OnboardingModalProps> = ({
  report,
  onAddWorkspaceAndScan,
  onFinishOnboarding,
}) => {
  const [step, setStep] = useState<1 | 2 | 3 | 4 | 5>(1);
  const [folderPath, setFolderPath] = useState('C:\\dev');
  const [isScanning, setIsScanning] = useState(false);
  const [discoveredCount, setDiscoveredCount] = useState<number>(0);
  const [chosenMode, setChosenMode] = useState<'safe' | 'integration'>('safe');

  const handleStartScan = async () => {
    setStep(3);
    setIsScanning(true);
    try {
      const count = await onAddWorkspaceAndScan(folderPath);
      setDiscoveredCount(count);
    } catch {
      setDiscoveredCount(0);
    } finally {
      setIsScanning(false);
    }
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.75)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 2000,
      backdropFilter: 'blur(4px)'
    }}>
      <div style={{
        width: '520px',
        backgroundColor: 'var(--bg-card)',
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-md)',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column'
      }}>
        {/* Step Indicator */}
        <div style={{
          display: 'flex',
          height: '4px',
          backgroundColor: 'var(--border-color)'
        }}>
          {[1, 2, 3, 4, 5].map(s => (
            <div
              key={s}
              style={{
                flex: 1,
                backgroundColor: s <= step ? 'var(--accent)' : 'transparent',
                transition: 'background-color 0.2s'
              }}
            />
          ))}
        </div>

        <div style={{ padding: '32px 36px', display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* STEP 1: Welcome */}
          {step === 1 && (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', gap: '16px' }}>
              <div style={{
                width: '64px',
                height: '64px',
                borderRadius: '16px',
                backgroundColor: 'var(--accent)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: '#fff',
                boxShadow: '0 8px 16px rgba(59, 130, 246, 0.3)'
              }}>
                <Compass size={36} />
              </div>
              <div>
                <h2 style={{ fontSize: '20px', fontWeight: 800 }}>Welcome to NodePilot</h2>
                <p style={{ fontSize: '14px', color: 'var(--text-secondary)', marginTop: '6px', maxWidth: '380px' }}>
                  Stop managing Node versions globally. Let each project define and run its own Node environment.
                </p>
              </div>
              <button
                onClick={() => setStep(2)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  padding: '10px 24px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: 'var(--accent)',
                  color: '#fff',
                  border: 'none',
                  fontWeight: 600,
                  fontSize: '14px',
                  cursor: 'pointer',
                  marginTop: '10px'
                }}
              >
                <span>Get Started</span>
                <ArrowRight size={16} />
              </button>
            </div>
          )}

          {/* STEP 2: Add development folder */}
          {step === 2 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                <div style={{
                  width: '40px',
                  height: '40px',
                  borderRadius: '10px',
                  backgroundColor: 'var(--accent-subtle)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: 'var(--accent)'
                }}>
                  <HardDrive size={22} />
                </div>
                <div>
                  <h3 style={{ fontSize: '16px', fontWeight: 700 }}>Add your development folder</h3>
                  <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
                    NodePilot will scan this directory to discover existing projects.
                  </p>
                </div>
              </div>

              <div>
                <label style={{ fontSize: '12px', fontWeight: 600, marginBottom: '6px', display: 'block' }}>
                  Workspace Path:
                </label>
                <input
                  type="text"
                  value={folderPath}
                  onChange={e => setFolderPath(e.target.value)}
                  placeholder="e.g. C:\dev or /Users/username/dev"
                  style={{
                    width: '100%',
                    padding: '10px 14px',
                    borderRadius: 'var(--radius-md)',
                    backgroundColor: 'var(--bg-sidebar)',
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-primary)',
                    fontSize: '13px',
                    fontFamily: 'var(--font-mono)'
                  }}
                />
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '10px', marginTop: '10px' }}>
                <button
                  onClick={() => setStep(1)}
                  style={{
                    padding: '8px 16px',
                    borderRadius: 'var(--radius-md)',
                    backgroundColor: 'transparent',
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-secondary)',
                    cursor: 'pointer'
                  }}
                >
                  Back
                </button>
                <button
                  onClick={handleStartScan}
                  disabled={!folderPath.trim()}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '8px 18px',
                    borderRadius: 'var(--radius-md)',
                    backgroundColor: 'var(--accent)',
                    color: '#fff',
                    border: 'none',
                    fontWeight: 600,
                    cursor: 'pointer'
                  }}
                >
                  <span>Scan Folder</span>
                  <ArrowRight size={15} />
                </button>
              </div>
            </div>
          )}

          {/* STEP 3: Scanning */}
          {step === 3 && (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', gap: '16px', padding: '10px 0' }}>
              {isScanning ? (
                <>
                  <RefreshCw size={36} color="var(--accent)" className="animate-spin" />
                  <div>
                    <h3 style={{ fontSize: '16px', fontWeight: 700 }}>Scanning repositories...</h3>
                    <p style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '4px' }}>
                      Inspecting package manifests and framework requirements passively
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <div style={{
                    width: '54px',
                    height: '54px',
                    borderRadius: '50%',
                    backgroundColor: 'var(--success-subtle)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    color: 'var(--success)'
                  }}>
                    <FolderGit2 size={28} />
                  </div>
                  <div>
                    <h3 style={{ fontSize: '17px', fontWeight: 700 }}>Scan Complete!</h3>
                    <p style={{ fontSize: '14px', color: 'var(--text-secondary)', marginTop: '4px' }}>
                      <span style={{ fontWeight: 700, color: 'var(--text-primary)' }}>{discoveredCount} projects</span> found in {folderPath}.
                    </p>
                  </div>
                  <button
                    onClick={() => setStep(4)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                      padding: '9px 20px',
                      borderRadius: 'var(--radius-md)',
                      backgroundColor: 'var(--accent)',
                      color: '#fff',
                      border: 'none',
                      fontWeight: 600,
                      cursor: 'pointer',
                      marginTop: '6px'
                    }}
                  >
                    <span>Next: Coexistence Check</span>
                    <ArrowRight size={15} />
                  </button>
                </>
              )}
            </div>
          )}

          {/* STEP 4: Coexistence Check */}
          {step === 4 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <ShieldCheck size={24} color="var(--success)" />
                <h3 style={{ fontSize: '16px', fontWeight: 700 }}>Existing Node Environment</h3>
              </div>

              <div style={{
                padding: '12px 14px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--bg-sidebar)',
                border: '1px solid var(--border-color)',
                fontSize: '12px',
                display: 'flex',
                flexDirection: 'column',
                gap: '4px'
              }}>
                <div>System Node: <strong>{report?.systemNodeVersion || 'Not in PATH'}</strong></div>
                {report?.existingManagers && report.existingManagers.length > 0 ? (
                  <div style={{ color: 'var(--warning)', marginTop: '2px' }}>
                    Detected managers: {report.existingManagers.map(m => m.name).join(', ')}
                  </div>
                ) : (
                  <div style={{ color: 'var(--success)' }}>No conflicting version managers detected.</div>
                )}
                <div style={{ color: 'var(--text-muted)', marginTop: '4px', fontSize: '11px' }}>
                  No files or settings were modified.
                </div>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginTop: '4px' }}>
                <label style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: '10px',
                  padding: '10px 12px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: chosenMode === 'safe' ? 'var(--accent-subtle)' : 'var(--bg-sidebar)',
                  border: `1px solid ${chosenMode === 'safe' ? 'var(--accent)' : 'var(--border-color)'}`,
                  cursor: 'pointer'
                }}>
                  <input
                    type="radio"
                    name="mode"
                    checked={chosenMode === 'safe'}
                    onChange={() => setChosenMode('safe')}
                    style={{ marginTop: '2px' }}
                  />
                  <div>
                    <span style={{ fontWeight: 600, fontSize: '13px' }}>Continue in Safe Mode (Recommended)</span>
                    <p style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
                      NodePilot operates without modifying your global PATH. Terminals launched from NodePilot use the assigned Node runtime.
                    </p>
                  </div>
                </label>

                <label style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: '10px',
                  padding: '10px 12px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: chosenMode === 'integration' ? 'var(--accent-subtle)' : 'var(--bg-sidebar)',
                  border: `1px solid ${chosenMode === 'integration' ? 'var(--accent)' : 'var(--border-color)'}`,
                  cursor: 'pointer'
                }}>
                  <input
                    type="radio"
                    name="mode"
                    checked={chosenMode === 'integration'}
                    onChange={() => setChosenMode('integration')}
                    style={{ marginTop: '2px' }}
                  />
                  <div>
                    <span style={{ fontWeight: 600, fontSize: '13px' }}>Enable Direct Shell Integration</span>
                    <p style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
                      Adds NodePilot shims to your user PATH so all terminals automatically resolve project Node versions.
                    </p>
                  </div>
                </label>
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '10px', marginTop: '8px' }}>
                <button
                  onClick={() => setStep(3)}
                  style={{
                    padding: '8px 14px',
                    borderRadius: 'var(--radius-md)',
                    backgroundColor: 'transparent',
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-secondary)',
                    cursor: 'pointer'
                  }}
                >
                  Back
                </button>
                <button
                  onClick={() => setStep(5)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '8px 18px',
                    borderRadius: 'var(--radius-md)',
                    backgroundColor: 'var(--accent)',
                    color: '#fff',
                    border: 'none',
                    fontWeight: 600,
                    cursor: 'pointer'
                  }}
                >
                  <span>Continue</span>
                  <ArrowRight size={15} />
                </button>
              </div>
            </div>
          )}

          {/* STEP 5: Ready */}
          {step === 5 && (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', gap: '16px', padding: '10px 0' }}>
              <div style={{
                width: '60px',
                height: '60px',
                borderRadius: '50%',
                backgroundColor: 'var(--success-subtle)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: 'var(--success)'
              }}>
                <CheckCircle2 size={34} />
              </div>
              <div>
                <h2 style={{ fontSize: '20px', fontWeight: 800 }}>You're all set!</h2>
                <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginTop: '4px', maxWidth: '360px' }}>
                  NodePilot is configured. Open your projects dashboard to view frameworks, assign Node versions, and launch terminals.
                </p>
              </div>
              <button
                onClick={() => onFinishOnboarding(chosenMode === 'integration')}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  padding: '10px 24px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: 'var(--accent)',
                  color: '#fff',
                  border: 'none',
                  fontWeight: 600,
                  fontSize: '14px',
                  cursor: 'pointer',
                  marginTop: '8px'
                }}
              >
                <span>Open Projects</span>
                <ArrowRight size={16} />
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
