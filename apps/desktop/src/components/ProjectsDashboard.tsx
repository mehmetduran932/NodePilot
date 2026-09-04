import React, { useState, useMemo } from 'react';
import { 
  Search, 
  Filter, 
  Terminal, 
  Code2, 
  FolderOpen, 
  CheckCircle2, 
  AlertTriangle, 
  HelpCircle,
  Sparkles,
  GitBranch,
  Layers,
  ArrowUpDown
} from 'lucide-react';
import { ProjectRecord, InstalledVersion } from '../types';

interface ProjectsDashboardProps {
  projects: ProjectRecord[];
  installedVersions: InstalledVersion[];
  onAssignNode: (projectPath: string, version: string) => void;
  onOpenTerminal: (projectPath: string) => void;
  onOpenEditor: (projectPath: string) => void;
  onOpenFolder: (projectPath: string) => void;
  onSelectProjectDetails: (project: ProjectRecord) => void;
  onOpenBulkAssign: (selectedPaths: string[]) => void;
  onOpenAutoAssignDiff: () => void;
  onAddWorkspaceClick: () => void;
}

export const ProjectsDashboard: React.FC<ProjectsDashboardProps> = ({
  projects,
  installedVersions,
  onAssignNode,
  onOpenTerminal,
  onOpenEditor,
  onOpenFolder,
  onSelectProjectDetails,
  onOpenBulkAssign,
  onOpenAutoAssignDiff,
  onAddWorkspaceClick,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [frameworkFilter, setFrameworkFilter] = useState('all');
  const [compatibilityFilter, setCompatibilityFilter] = useState('all');
  const [sortBy, setSortBy] = useState<'name' | 'framework' | 'node' | 'compat'>('name');
  const [sortAsc, setSortAsc] = useState(true);
  const [selectedProjectPaths, setSelectedProjectPaths] = useState<string[]>([]);

  // Unique framework list for filter dropdown
  const frameworks = useMemo(() => {
    const set = new Set<string>();
    projects.forEach(p => set.add(p.framework));
    return Array.from(set).sort();
  }, [projects]);

  // Filtered & sorted projects
  const filteredProjects = useMemo(() => {
    return projects
      .filter(p => {
        const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase())
          || p.path.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesFw = frameworkFilter === 'all' || p.framework === frameworkFilter;
        const matchesCompat = compatibilityFilter === 'all' || p.compatibilityStatus === compatibilityFilter;
        return matchesSearch && matchesFw && matchesCompat;
      })
      .sort((a, b) => {
        let cmp = 0;
        if (sortBy === 'name') cmp = a.name.localeCompare(b.name);
        else if (sortBy === 'framework') cmp = a.framework.localeCompare(b.framework);
        else if (sortBy === 'node') cmp = (a.assignedNode || '').localeCompare(b.assignedNode || '');
        else if (sortBy === 'compat') cmp = a.compatibilityStatus.localeCompare(b.compatibilityStatus);
        return sortAsc ? cmp : -cmp;
      });
  }, [projects, searchQuery, frameworkFilter, compatibilityFilter, sortBy, sortAsc]);

  const toggleSelectAll = () => {
    if (selectedProjectPaths.length === filteredProjects.length) {
      setSelectedProjectPaths([]);
    } else {
      setSelectedProjectPaths(filteredProjects.map(p => p.path));
    }
  };

  const toggleSelectProject = (path: string) => {
    if (selectedProjectPaths.includes(path)) {
      setSelectedProjectPaths(selectedProjectPaths.filter(p => p !== path));
    } else {
      setSelectedProjectPaths([...selectedProjectPaths, path]);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Top Action & Filter Header */}
      <div style={{
        padding: '16px 24px',
        borderBottom: '1px solid var(--border-color)',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
        backgroundColor: 'var(--bg-app)'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 700 }}>Projects</h2>
            <p style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
              {filteredProjects.length} {filteredProjects.length === 1 ? 'project' : 'projects'} found
            </p>
          </div>

          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={onOpenAutoAssignDiff}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '7px 12px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--accent-subtle)',
                border: '1px solid var(--accent)',
                color: 'var(--accent)',
                cursor: 'pointer',
                fontWeight: 600,
                fontSize: '12px'
              }}
            >
              <Sparkles size={14} />
              <span>Auto-Assign Recommended</span>
            </button>
          </div>
        </div>

        {/* Search & Filter Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
          {/* Search Box */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            padding: '6px 10px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--bg-card)',
            border: '1px solid var(--border-color)',
            flex: '1',
            minWidth: '220px'
          }}>
            <Search size={15} color="var(--text-muted)" />
            <input
              type="text"
              placeholder="Search projects..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              style={{
                background: 'transparent',
                border: 'none',
                outline: 'none',
                width: '100%',
                color: 'var(--text-primary)',
                fontSize: '12px'
              }}
            />
          </div>

          {/* Framework Filter */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <Filter size={14} color="var(--text-muted)" />
            <select
              value={frameworkFilter}
              onChange={e => setFrameworkFilter(e.target.value)}
              style={{
                padding: '6px 10px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--bg-card)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                fontSize: '12px',
                cursor: 'pointer'
              }}
            >
              <option value="all">All Frameworks</option>
              {frameworks.map(fw => (
                <option key={fw} value={fw}>{fw}</option>
              ))}
            </select>
          </div>

          {/* Compatibility Filter */}
          <select
            value={compatibilityFilter}
            onChange={e => setCompatibilityFilter(e.target.value)}
            style={{
              padding: '6px 10px',
              borderRadius: 'var(--radius-md)',
              backgroundColor: 'var(--bg-card)',
              border: '1px solid var(--border-color)',
              color: 'var(--text-primary)',
              fontSize: '12px',
              cursor: 'pointer'
            }}
          >
            <option value="all">All Statuses</option>
            <option value="compatible">Compatible</option>
            <option value="incompatible">Incompatible</option>
            <option value="unknown">Unknown</option>
          </select>

          {/* Sort Dropdown */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <select
              value={sortBy}
              onChange={e => setSortBy(e.target.value as any)}
              style={{
                padding: '6px 8px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--bg-card)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                fontSize: '12px',
                cursor: 'pointer'
              }}
            >
              <option value="name">Sort by Name</option>
              <option value="framework">Sort by Framework</option>
              <option value="node">Sort by Node</option>
              <option value="compat">Sort by Status</option>
            </select>
            <button
              onClick={() => setSortAsc(!sortAsc)}
              title={sortAsc ? 'Ascending' : 'Descending'}
              style={{
                padding: '6px 8px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--bg-card)',
                border: '1px solid var(--border-color)',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center'
              }}
            >
              <ArrowUpDown size={13} color="var(--text-secondary)" />
            </button>
          </div>
        </div>

        {/* Bulk Selection Bar */}
        {selectedProjectPaths.length > 0 && (
          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '8px 12px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--accent-subtle)',
            border: '1px solid var(--accent)'
          }}>
            <span style={{ fontSize: '12px', fontWeight: 600, color: 'var(--accent)' }}>
              {selectedProjectPaths.length} projects selected
            </span>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={() => onOpenBulkAssign(selectedProjectPaths)}
                style={{
                  padding: '5px 12px',
                  borderRadius: 'var(--radius-sm)',
                  backgroundColor: 'var(--accent)',
                  color: '#fff',
                  border: 'none',
                  fontSize: '12px',
                  fontWeight: 600,
                  cursor: 'pointer'
                }}
              >
                Assign Node.js Version...
              </button>
              <button
                onClick={() => setSelectedProjectPaths([])}
                style={{
                  padding: '5px 8px',
                  borderRadius: 'var(--radius-sm)',
                  backgroundColor: 'transparent',
                  color: 'var(--text-secondary)',
                  border: 'none',
                  fontSize: '12px',
                  cursor: 'pointer'
                }}
              >
                Deselect
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Main Content Area */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '16px 24px' }}>
        {projects.length === 0 ? (
          /* Empty State: No projects discovered */
          <div style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            height: '70%',
            textAlign: 'center',
            gap: '14px'
          }}>
            <div style={{
              width: '56px',
              height: '56px',
              borderRadius: '50%',
              backgroundColor: 'var(--border-color)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center'
            }}>
              <Layers size={28} color="var(--text-muted)" />
            </div>
            <div>
              <h3 style={{ fontSize: '16px', fontWeight: 600 }}>No projects yet</h3>
              <p style={{ fontSize: '13px', color: 'var(--text-muted)', maxWidth: '360px', marginTop: '4px' }}>
                Add the folder where you keep your development repositories to automatically discover your projects.
              </p>
            </div>
            <button
              onClick={onAddWorkspaceClick}
              style={{
                padding: '9px 18px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--accent)',
                color: '#fff',
                border: 'none',
                fontWeight: 600,
                cursor: 'pointer',
                fontSize: '13px'
              }}
            >
              Add Workspace Folder
            </button>
          </div>
        ) : filteredProjects.length === 0 ? (
          /* Empty Filter State */
          <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-muted)' }}>
            No projects match the selected filters.
          </div>
        ) : (
          /* Projects Table */
          <div style={{
            borderRadius: 'var(--radius-md)',
            border: '1px solid var(--border-color)',
            overflow: 'hidden',
            backgroundColor: 'var(--bg-card)'
          }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
              <thead>
                <tr style={{
                  borderBottom: '1px solid var(--border-color)',
                  backgroundColor: 'var(--bg-sidebar)',
                  fontSize: '11px',
                  textTransform: 'uppercase',
                  letterSpacing: '0.5px',
                  color: 'var(--text-muted)'
                }}>
                  <th style={{ width: '38px', padding: '10px 12px' }}>
                    <input
                      type="checkbox"
                      checked={selectedProjectPaths.length === filteredProjects.length && filteredProjects.length > 0}
                      onChange={toggleSelectAll}
                      style={{ cursor: 'pointer' }}
                    />
                  </th>
                  <th style={{ padding: '10px 12px' }}>Project</th>
                  <th style={{ padding: '10px 12px' }}>Framework</th>
                  <th style={{ padding: '10px 12px' }}>Node Version</th>
                  <th style={{ padding: '10px 12px' }}>Status</th>
                  <th style={{ padding: '10px 12px', textAlign: 'right' }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredProjects.map(proj => {
                  const isSelected = selectedProjectPaths.includes(proj.path);
                  return (
                    <tr
                      key={proj.id}
                      style={{
                        borderBottom: '1px solid var(--border-color)',
                        backgroundColor: isSelected ? 'var(--bg-card-selected)' : 'transparent',
                        transition: 'background-color 0.1s'
                      }}
                    >
                      {/* Checkbox */}
                      <td style={{ padding: '10px 12px' }}>
                        <input
                          type="checkbox"
                          checked={isSelected}
                          onChange={() => toggleSelectProject(proj.path)}
                          style={{ cursor: 'pointer' }}
                        />
                      </td>

                      {/* Project Name & Path */}
                      <td style={{ padding: '10px 12px' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                          <span
                            onClick={() => onSelectProjectDetails(proj)}
                            style={{ fontWeight: 600, color: 'var(--text-primary)', cursor: 'pointer' }}
                            title="Click for details"
                          >
                            {proj.name}
                          </span>
                          {proj.hasGit && (
                            <span title="Git Repository">
                              <GitBranch size={13} color="var(--text-muted)" />
                            </span>
                          )}
                        </div>
                        <div style={{ fontSize: '11px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
                          {proj.path}
                        </div>
                      </td>

                      {/* Framework */}
                      <td style={{ padding: '10px 12px' }}>
                        <span style={{
                          padding: '3px 8px',
                          borderRadius: '4px',
                          backgroundColor: 'var(--border-color)',
                          fontSize: '12px',
                          fontWeight: 500
                        }}>
                          {proj.framework} {proj.frameworkVersion || ''}
                        </span>
                      </td>

                      {/* Node Version Dropdown */}
                      <td style={{ padding: '10px 12px' }}>
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '3px' }}>
                          <select
                            value={proj.assignedNode || ''}
                            onChange={e => onAssignNode(proj.path, e.target.value)}
                            style={{
                              padding: '5px 8px',
                              borderRadius: 'var(--radius-sm)',
                              backgroundColor: 'var(--bg-sidebar)',
                              border: '1px solid var(--border-color)',
                              color: 'var(--text-primary)',
                              fontSize: '12px',
                              cursor: 'pointer',
                              fontWeight: 500
                            }}
                          >
                            <option value="" disabled>Select version...</option>
                            <optgroup label="Installed Runtimes">
                              {installedVersions.map(iv => (
                                <option key={iv.version} value={iv.version}>
                                  v{iv.version}
                                </option>
                              ))}
                            </optgroup>
                            <optgroup label="Available LTS">
                              <option value="22.18.0">22.18.0 (LTS Iron)</option>
                              <option value="20.19.5">20.19.5 (LTS Hydrogen)</option>
                              <option value="18.20.8">18.20.8 (LTS Gallium)</option>
                              <option value="16.20.2">16.20.2 (LTS Fermium)</option>
                            </optgroup>
                          </select>
                          <span style={{ fontSize: '10px', color: 'var(--text-muted)' }}>
                            Source: {proj.isInherited ? 'Parent inheritance' : proj.configSource}
                          </span>
                        </div>
                      </td>

                      {/* Compatibility Status */}
                      <td style={{ padding: '10px 12px' }}>
                        {proj.compatibilityStatus === 'compatible' ? (
                          <div style={{ display: 'flex', alignItems: 'center', gap: '5px', color: 'var(--success)' }}>
                            <CheckCircle2 size={15} />
                            <span style={{ fontSize: '12px', fontWeight: 500 }}>Compatible</span>
                          </div>
                        ) : proj.compatibilityStatus === 'incompatible' ? (
                          <div
                            style={{ display: 'flex', alignItems: 'center', gap: '5px', color: 'var(--danger)', cursor: 'help' }}
                            title={proj.compatibilityMessage || 'Node version incompatible with framework'}
                          >
                            <AlertTriangle size={15} />
                            <span style={{ fontSize: '12px', fontWeight: 500 }}>Incompatible</span>
                          </div>
                        ) : (
                          <div style={{ display: 'flex', alignItems: 'center', gap: '5px', color: 'var(--text-muted)' }}>
                            <HelpCircle size={15} />
                            <span style={{ fontSize: '12px' }}>Unassigned</span>
                          </div>
                        )}
                      </td>

                      {/* Actions */}
                      <td style={{ padding: '10px 12px', textAlign: 'right' }}>
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'flex-end', gap: '6px' }}>
                          <button
                            onClick={() => onOpenTerminal(proj.path)}
                            title="Open Terminal with Project Node"
                            style={{
                              padding: '5px 8px',
                              borderRadius: 'var(--radius-sm)',
                              backgroundColor: 'var(--border-color)',
                              border: 'none',
                              cursor: 'pointer'
                            }}
                          >
                            <Terminal size={14} color="var(--text-primary)" />
                          </button>

                          <button
                            onClick={() => onOpenEditor(proj.path)}
                            title="Open in Code Editor"
                            style={{
                              padding: '5px 8px',
                              borderRadius: 'var(--radius-sm)',
                              backgroundColor: 'var(--border-color)',
                              border: 'none',
                              cursor: 'pointer'
                            }}
                          >
                            <Code2 size={14} color="var(--text-primary)" />
                          </button>

                          <button
                            onClick={() => onOpenFolder(proj.path)}
                            title="Open Folder"
                            style={{
                              padding: '5px 8px',
                              borderRadius: 'var(--radius-sm)',
                              backgroundColor: 'var(--border-color)',
                              border: 'none',
                              cursor: 'pointer'
                            }}
                          >
                            <FolderOpen size={14} color="var(--text-primary)" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
