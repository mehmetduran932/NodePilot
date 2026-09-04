export type PackageManager = 'npm' | 'pnpm' | 'yarn' | 'bun' | 'unknown';

export type CompatibilityStatus = 'compatible' | 'incompatible' | 'unknown';

export type ConfigSourceType =
  | 'localOverride'
  | 'nvmrc'
  | 'nodeVersion'
  | 'volta'
  | 'engines'
  | 'toolVersions'
  | 'mise'
  | 'inheritedLocal'
  | 'inheritedOther'
  | 'globalDefault'
  | 'none';

export interface ProjectRecord {
  id: string;
  name: string;
  path: string;
  workspaceRoot: string;
  framework: string;
  frameworkVersion?: string;
  packageManager: PackageManager;
  nodeRequirements?: string;
  assignedNode?: string;
  configSource: ConfigSourceType;
  configSourcePath?: string;
  isInherited: boolean;
  compatibilityStatus: CompatibilityStatus;
  compatibilityMessage?: string;
  recommendedNode?: string;
  hasGit: boolean;
  isMonorepoRoot: boolean;
  monorepoRootPath?: string;
  lastScanned: string;
}

export interface AppState {
  workspaces: string[];
  projects: ProjectRecord[];
  lastScanTime?: string;
}

export interface Settings {
  scanOnStartup: boolean;
  autoCheckNodeUpdates: boolean;
  autoCheckAppUpdates: boolean;
  showPrereleases: boolean;
  preferredTerminal: string;
  preferredEditor: string;
  safeMode: boolean;
  shellIntegrationEnabled: boolean;
  telemetry: boolean;
  theme: string;
  globalDefaultVersion?: string;
}

export interface InstalledVersion {
  version: string;
  path: string;
  binaryPath: string;
  npmVersion?: string;
  sizeBytes: number;
  projectsUsingCount: number;
  projectsUsing: string[];
}

export interface NodeRelease {
  version: string;
  date: string;
  files: string[];
  npm?: string;
  lts: boolean | string;
  security: boolean;
}

export interface DetectedManager {
  name: string;
  description: string;
  path?: string;
  activeInPath: boolean;
  version?: string;
}

export interface EnvironmentReport {
  existingManagers: DetectedManager[];
  systemNodePath?: string;
  systemNodeVersion?: string;
  hasPotentialConflicts: boolean;
  pathEntries: string[];
  recommendedMode: string;
}

export interface IntegrationStatus {
  isEnabled: boolean;
  binDir: string;
  inCurrentProcessPath: boolean;
  inPersistedUserPath: boolean;
  backupAvailable: boolean;
}

export interface AutoAssignProposal {
  projectId: string;
  projectName: string;
  currentVersion?: string;
  proposedVersion: string;
  reason: string;
}

export interface UpdateCheckResult {
  updateAvailable: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseNotes?: string;
  downloadUrl?: string;
  packageManagerCommand: string;
}
