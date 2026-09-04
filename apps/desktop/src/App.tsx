import React, { useState, useEffect } from 'react';
import { Navigation, NavTab } from './components/Navigation';
import { ProjectsDashboard } from './components/ProjectsDashboard';
import { ProjectDetailsModal } from './components/ProjectDetailsModal';
import { BulkAssignModal } from './components/BulkAssignModal';
import { AutoAssignDiffModal } from './components/AutoAssignDiffModal';
import { NodeManagerView } from './components/NodeManagerView';
import { WorkspacesView } from './components/WorkspacesView';
import { DoctorView } from './components/DoctorView';
import { SettingsView } from './components/SettingsView';
import { OnboardingModal } from './components/OnboardingModal';
import { 
  ProjectRecord, 
  InstalledVersion, 
  NodeRelease, 
  EnvironmentReport, 
  IntegrationStatus, 
  Settings, 
  AutoAssignProposal, 
  UpdateCheckResult 
} from './types';

// Safely invoke Tauri commands or HTTP API when running in browser mode
async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  if (isTauri) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(cmd, args);
    } catch (err) {
      console.warn(`Tauri invoke ${cmd} error:`, err);
      throw err;
    }
  }

  // Real HTTP API server in browser mode
  try {
    const res = await fetch('/api/invoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ cmd, args: args || {} }),
    });
    if (res.ok) {
      return await res.json() as T;
    }
  } catch (err) {
    // API server not responding, fallback to mock
  }

  // Browser preview fallback mock
  return mockTauriCall<T>(cmd, args);
}

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<NavTab>('projects');
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [workspaces, setWorkspaces] = useState<string[]>([]);
  const [lastScanTime, setLastScanTime] = useState<string | undefined>();
  const [installedVersions, setInstalledVersions] = useState<InstalledVersion[]>([]);
  const [availableReleases, setAvailableReleases] = useState<NodeRelease[]>([]);
  const [settings, setSettings] = useState<Settings>({
    scanOnStartup: true,
    autoCheckNodeUpdates: true,
    autoCheckAppUpdates: true,
    showPrereleases: false,
    preferredTerminal: 'system',
    preferredEditor: 'code',
    safeMode: true,
    shellIntegrationEnabled: false,
    telemetry: false,
    theme: 'dark',
  });
  const [doctorReport, setDoctorReport] = useState<EnvironmentReport | null>(null);
  const [integrationStatus, setIntegrationStatus] = useState<IntegrationStatus | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateCheckResult | null>(null);

  // Loading & Progress States
  const [isScanning, setIsScanning] = useState(false);
  const [isInstallingNode, setIsInstallingNode] = useState(false);
  const [installingVersionName, setInstallingVersionName] = useState<string | null>(null);
  const [isTogglingIntegration, setIsTogglingIntegration] = useState(false);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);

  // Modals
  const [selectedDetailsProject, setSelectedDetailsProject] = useState<ProjectRecord | null>(null);
  const [bulkAssignPaths, setBulkAssignPaths] = useState<string[] | null>(null);
  const [autoAssignDiff, setAutoAssignDiff] = useState<AutoAssignProposal[] | null>(null);
  const [showOnboarding, setShowOnboarding] = useState(false);

  // Apply Theme
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', settings.theme);
  }, [settings.theme]);

  // Initial Load
  useEffect(() => {
    loadInitialData();
  }, []);

  const loadInitialData = async () => {
    try {
      const state = await tauriInvoke<{ workspaces: string[]; projects: ProjectRecord[]; lastScanTime?: string }>('get_state');
      setWorkspaces(state.workspaces || []);
      setProjects(state.projects || []);
      setLastScanTime(state.lastScanTime);

      if ((!state.workspaces || state.workspaces.length === 0) && (!state.projects || state.projects.length === 0)) {
        setShowOnboarding(true);
      }

      const st = await tauriInvoke<Settings>('get_settings');
      setSettings(st);

      const installed = await tauriInvoke<InstalledVersion[]>('get_installed_node_versions');
      setInstalledVersions(installed || []);

      const report = await tauriInvoke<EnvironmentReport>('get_doctor_report');
      setDoctorReport(report);

      const integ = await tauriInvoke<IntegrationStatus>('get_shell_integration_status');
      setIntegrationStatus(integ);

      // Fetch available releases in background
      tauriInvoke<NodeRelease[]>('get_available_node_versions')
        .then(rel => setAvailableReleases(rel || []))
        .catch(() => {});
    } catch (err) {
      console.error('Failed to load initial data:', err);
    }
  };

  // Workspaces & Scanning
  const handleScanWorkspaces = async () => {
    setIsScanning(true);
    try {
      const updated = await tauriInvoke<ProjectRecord[]>('scan_workspaces');
      setProjects(updated);
      setLastScanTime(new Date().toISOString());
      // Also refresh installed counts
      const installed = await tauriInvoke<InstalledVersion[]>('get_installed_node_versions');
      setInstalledVersions(installed || []);
    } catch (err) {
      console.error('Scan failed:', err);
    } finally {
      setIsScanning(false);
    }
  };

  const handleAddWorkspace = async (path: string) => {
    try {
      const state = await tauriInvoke<{ workspaces: string[]; projects: ProjectRecord[] }>('add_workspace', { path });
      setWorkspaces(state.workspaces);
      handleScanWorkspaces();
    } catch (err) {
      console.error('Add workspace failed:', err);
    }
  };

  const handleRemoveWorkspace = async (path: string) => {
    try {
      const state = await tauriInvoke<{ workspaces: string[]; projects: ProjectRecord[] }>('remove_workspace', { path });
      setWorkspaces(state.workspaces);
      setProjects(state.projects);
    } catch (err) {
      console.error('Remove workspace failed:', err);
    }
  };

  // Node Assignment
  const handleAssignNode = async (projectPath: string, version: string) => {
    try {
      const updated = await tauriInvoke<ProjectRecord>('assign_project_node', { projectPath, version });
      setProjects(prev => prev.map(p => p.path === projectPath ? updated : p));
      if (selectedDetailsProject?.path === projectPath) {
        setSelectedDetailsProject(updated);
      }
      // Refresh installed usage
      const installed = await tauriInvoke<InstalledVersion[]>('get_installed_node_versions');
      setInstalledVersions(installed || []);
    } catch (err) {
      console.error('Assign failed:', err);
    }
  };

  const handleBulkAssign = async (paths: string[], version: string) => {
    try {
      await tauriInvoke<number>('bulk_assign_node', { projectPaths: paths, version });
      setBulkAssignPaths(null);
      handleScanWorkspaces();
    } catch (err) {
      console.error('Bulk assign failed:', err);
    }
  };

  const handleOpenAutoAssignDiff = async () => {
    try {
      const diff = await tauriInvoke<AutoAssignProposal[]>('get_auto_assign_diff');
      setAutoAssignDiff(diff);
    } catch (err) {
      console.error('Fetch diff failed:', err);
    }
  };

  const handleApplyAutoAssignAll = async () => {
    try {
      await tauriInvoke<number>('apply_auto_assign_recommendations');
      setAutoAssignDiff(null);
      handleScanWorkspaces();
    } catch (err) {
      console.error('Apply all auto-assign failed:', err);
    }
  };

  // Node Runtime Installation
  const handleInstallNodeVersion = async (version: string) => {
    setIsInstallingNode(true);
    setInstallingVersionName(version);
    try {
      await tauriInvoke<string>('install_node_version', { version });
      const installed = await tauriInvoke<InstalledVersion[]>('get_installed_node_versions');
      setInstalledVersions(installed || []);
    } catch (err) {
      console.error('Install node failed:', err);
    } finally {
      setIsInstallingNode(false);
      setInstallingVersionName(null);
    }
  };

  const handleUninstallNodeVersion = async (version: string) => {
    try {
      await tauriInvoke<void>('uninstall_node_version_cmd', { version });
      const installed = await tauriInvoke<InstalledVersion[]>('get_installed_node_versions');
      setInstalledVersions(installed || []);
    } catch (err) {
      console.error('Uninstall failed:', err);
    }
  };

  // Actions
  const handleOpenTerminal = async (projectPath: string) => {
    try {
      await tauriInvoke<void>('open_project_terminal', { projectPath });
    } catch (err) {
      console.error('Terminal launch failed:', err);
    }
  };

  const handleOpenEditor = async (projectPath: string) => {
    try {
      await tauriInvoke<void>('open_project_editor', { projectPath, editor: settings.preferredEditor });
    } catch (err) {
      console.error('Editor launch failed:', err);
    }
  };

  const handleOpenFolder = async (projectPath: string) => {
    try {
      await tauriInvoke<void>('open_project_folder', { projectPath });
    } catch (err) {
      console.error('Folder open failed:', err);
    }
  };

  const handleToggleIntegration = async (enable: boolean) => {
    setIsTogglingIntegration(true);
    try {
      const updated = await tauriInvoke<IntegrationStatus>('toggle_shell_integration', { enable });
      setIntegrationStatus(updated);
    } catch (err) {
      console.error('Toggle integration failed:', err);
    } finally {
      setIsTogglingIntegration(false);
    }
  };

  const handleCheckAppUpdate = async () => {
    setIsCheckingUpdate(true);
    try {
      const res = await tauriInvoke<UpdateCheckResult>('check_app_updates');
      setUpdateInfo(res);
    } catch (err) {
      console.error('Check update failed:', err);
    } finally {
      setIsCheckingUpdate(false);
    }
  };

  // Onboarding Helper
  const handleAddWorkspaceAndScan = async (path: string): Promise<number> => {
    const state = await tauriInvoke<{ workspaces: string[]; projects: ProjectRecord[] }>('add_workspace', { path });
    setWorkspaces(state.workspaces);
    const scanned = await tauriInvoke<ProjectRecord[]>('scan_workspaces');
    setProjects(scanned);
    return scanned.length;
  };

  const handleFinishOnboarding = async (enableIntegration: boolean) => {
    setShowOnboarding(false);
    if (enableIntegration) {
      handleToggleIntegration(true);
    }
  };

  return (
    <div style={{ display: 'flex', width: '100vw', height: '100vh', overflow: 'hidden' }}>
      {/* Sidebar Navigation */}
      <Navigation
        currentTab={currentTab}
        onSelectTab={setCurrentTab}
        projectCount={projects.length}
        installedCount={installedVersions.length}
        onQuickScan={handleScanWorkspaces}
        isScanning={isScanning}
      />

      {/* Main Content Body */}
      <main style={{ flex: 1, height: '100vh', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
        {currentTab === 'projects' && (
          <ProjectsDashboard
            projects={projects}
            installedVersions={installedVersions}
            onAssignNode={handleAssignNode}
            onOpenTerminal={handleOpenTerminal}
            onOpenEditor={handleOpenEditor}
            onOpenFolder={handleOpenFolder}
            onSelectProjectDetails={setSelectedDetailsProject}
            onOpenBulkAssign={setBulkAssignPaths}
            onOpenAutoAssignDiff={handleOpenAutoAssignDiff}
            onAddWorkspaceClick={() => setCurrentTab('workspaces')}
          />
        )}

        {currentTab === 'node' && (
          <NodeManagerView
            installedVersions={installedVersions}
            availableReleases={availableReleases}
            isInstalling={isInstallingNode}
            installingVersion={installingVersionName}
            onInstallVersion={handleInstallNodeVersion}
            onUninstallVersion={handleUninstallNodeVersion}
            onRefreshReleases={() => {
              tauriInvoke<NodeRelease[]>('get_available_node_versions').then(setAvailableReleases);
            }}
          />
        )}

        {currentTab === 'workspaces' && (
          <WorkspacesView
            workspaces={workspaces}
            projects={projects}
            lastScanTime={lastScanTime}
            isScanning={isScanning}
            onAddWorkspace={handleAddWorkspace}
            onRemoveWorkspace={handleRemoveWorkspace}
            onScanNow={handleScanWorkspaces}
          />
        )}

        {currentTab === 'doctor' && (
          <DoctorView
            report={doctorReport}
            integration={integrationStatus}
            isTogglingIntegration={isTogglingIntegration}
            onToggleIntegration={handleToggleIntegration}
            onRefreshDoctor={() => {
              tauriInvoke<EnvironmentReport>('get_doctor_report').then(setDoctorReport);
              tauriInvoke<IntegrationStatus>('get_shell_integration_status').then(setIntegrationStatus);
            }}
          />
        )}

        {currentTab === 'settings' && (
          <SettingsView
            settings={settings}
            onUpdateSettings={async newSt => {
              setSettings(newSt);
              await tauriInvoke('update_settings', { settings: newSt });
            }}
            updateInfo={updateInfo}
            isCheckingUpdate={isCheckingUpdate}
            onCheckAppUpdate={handleCheckAppUpdate}
          />
        )}
      </main>

      {/* Project Details Modal */}
      {selectedDetailsProject && (
        <ProjectDetailsModal
          project={selectedDetailsProject}
          installedVersions={installedVersions}
          onClose={() => setSelectedDetailsProject(null)}
          onAssignNode={handleAssignNode}
          onOpenTerminal={handleOpenTerminal}
          onOpenEditor={handleOpenEditor}
          onOpenFolder={handleOpenFolder}
          onRemoveAssignment={async (path) => {
            handleAssignNode(path, '');
            setSelectedDetailsProject(null);
          }}
        />
      )}

      {/* Bulk Assign Modal */}
      {bulkAssignPaths && (
        <BulkAssignModal
          selectedPaths={bulkAssignPaths}
          installedVersions={installedVersions}
          onClose={() => setBulkAssignPaths(null)}
          onApplyBulkAssign={handleBulkAssign}
        />
      )}

      {/* Auto-Assign Diff Preview Modal */}
      {autoAssignDiff && (
        <AutoAssignDiffModal
          diff={autoAssignDiff}
          onClose={() => setAutoAssignDiff(null)}
          onApplyAll={handleApplyAutoAssignAll}
        />
      )}

      {/* Onboarding Modal */}
      {showOnboarding && (
        <OnboardingModal
          report={doctorReport}
          onAddWorkspaceAndScan={handleAddWorkspaceAndScan}
          onFinishOnboarding={handleFinishOnboarding}
        />
      )}
    </div>
  );
};

// Clean mock data for development preview when not running inside native Tauri runtime
function mockTauriCall<T>(cmd: string, _args?: Record<string, unknown>): Promise<T> {
  if (cmd === 'get_state') {
    return Promise.resolve({
      workspaces: ['C:\\dev'],
      projects: [
        {
          id: 'proj1',
          name: 'legacy-admin',
          path: 'C:\\dev\\legacy-admin',
          workspaceRoot: 'C:\\dev',
          framework: 'Angular',
          frameworkVersion: '13.3.12',
          packageManager: 'npm',
          assignedNode: '16.20.2',
          configSource: 'localOverride',
          isInherited: false,
          compatibilityStatus: 'compatible',
          compatibilityMessage: 'Node 16.20.2 is compatible with Angular 13',
          hasGit: true,
          isMonorepoRoot: false,
          lastScanned: new Date().toISOString(),
        },
        {
          id: 'proj2',
          name: 'lcm-web',
          path: 'C:\\dev\\lcm-web',
          workspaceRoot: 'C:\\dev',
          framework: 'Angular',
          frameworkVersion: '22.1.3',
          packageManager: 'npm',
          assignedNode: '22.18.0',
          configSource: 'localOverride',
          isInherited: false,
          compatibilityStatus: 'compatible',
          compatibilityMessage: 'Node 22.18.0 is compatible with Angular 22',
          hasGit: true,
          isMonorepoRoot: false,
          lastScanned: new Date().toISOString(),
        },
        {
          id: 'proj3',
          name: 'cv-builder',
          path: 'C:\\dev\\cv-builder',
          workspaceRoot: 'C:\\dev',
          framework: 'Next.js',
          frameworkVersion: '14.2.14',
          packageManager: 'pnpm',
          assignedNode: '20.19.5',
          configSource: 'nvmrc',
          isInherited: false,
          compatibilityStatus: 'compatible',
          compatibilityMessage: 'Node 20.19.5 is compatible with Next.js 14',
          hasGit: true,
          isMonorepoRoot: false,
          lastScanned: new Date().toISOString(),
        },
        {
          id: 'proj4',
          name: 'old-dashboard',
          path: 'C:\\dev\\old-dashboard',
          workspaceRoot: 'C:\\dev',
          framework: 'Angular',
          frameworkVersion: '11.2.0',
          packageManager: 'npm',
          assignedNode: '22.18.0',
          configSource: 'globalDefault',
          isInherited: false,
          compatibilityStatus: 'incompatible',
          compatibilityMessage: 'Node 22.18.0 is incompatible with Angular 11. Angular 11 requires Node 10.13 - 12.x or 14.x',
          recommendedNode: '14.21.3',
          hasGit: true,
          isMonorepoRoot: false,
          lastScanned: new Date().toISOString(),
        }
      ],
      lastScanTime: new Date().toISOString(),
    } as unknown as T);
  }

  if (cmd === 'get_installed_node_versions') {
    return Promise.resolve([
      { version: '22.18.0', path: 'C:\\Users\\dev\\AppData\\Local\\NodePilot\\versions\\22.18.0', binaryPath: 'node.exe', sizeBytes: 88080384, projectsUsingCount: 14, projectsUsing: ['lcm-web', 'old-dashboard'] },
      { version: '20.19.5', path: 'C:\\Users\\dev\\AppData\\Local\\NodePilot\\versions\\20.19.5', binaryPath: 'node.exe', sizeBytes: 81788928, projectsUsingCount: 8, projectsUsing: ['cv-builder'] },
      { version: '16.20.2', path: 'C:\\Users\\dev\\AppData\\Local\\NodePilot\\versions\\16.20.2', binaryPath: 'node.exe', sizeBytes: 74448896, projectsUsingCount: 4, projectsUsing: ['legacy-admin'] },
    ] as unknown as T);
  }

  if (cmd === 'get_doctor_report') {
    return Promise.resolve({
      existingManagers: [
        { name: 'fnm', description: 'Fast Node Manager', activeInPath: true },
        { name: 'nvm-windows', description: 'Node Version Manager for Windows', activeInPath: true },
      ],
      systemNodePath: 'C:\\Program Files\\nodejs\\node.exe',
      systemNodeVersion: 'v22.18.0',
      hasPotentialConflicts: true,
      pathEntries: [],
      recommendedMode: 'Safe Coexistence Mode',
    } as unknown as T);
  }

  if (cmd === 'get_shell_integration_status') {
    return Promise.resolve({
      isEnabled: false,
      binDir: 'C:\\Users\\dev\\AppData\\Local\\NodePilot\\bin',
      inCurrentProcessPath: false,
      inPersistedUserPath: false,
      backupAvailable: false,
    } as unknown as T);
  }

  return Promise.resolve({} as unknown as T);
}
