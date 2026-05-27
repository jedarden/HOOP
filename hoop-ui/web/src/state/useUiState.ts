import { useEffect, useState, useCallback } from 'react';
import { atom, useAtom } from 'jotai';

// UI state types matching the schema in hoop-schema/schemas/ui_state.json
export interface UiStateFilters {
  bead_status?: ('open' | 'closed')[];
  stitch_kind?: ('operator' | 'dictated' | 'worker' | 'ad-hoc')[];
  conversation_filter?: 'all' | 'fleet' | 'operator' | 'ad-hoc' | 'dictated';
  show_archived?: boolean;
}

export interface UiStatePanelLayout {
  fleet_visible?: boolean;
  beads_visible?: boolean;
  conversations_visible?: boolean;
  patterns_visible?: boolean;
}

export interface UiState {
  pinned_projects?: string[];
  active_project?: string | null;
  active_stitch?: string | null;
  sidebar_width?: number;
  panel_layout?: UiStatePanelLayout;
  filters?: UiStateFilters;
  theme?: 'light' | 'dark' | 'auto';
  schema_version: string;
}

interface UiStateResponse {
  schema_version: string;
  state: Record<string, string>;
  operator_id: string;
}

// Base atom for UI state
const baseUiStateAtom = atom<UiState>({
  schema_version: '1.1.0',
  pinned_projects: [],
  active_project: null,
  active_stitch: null,
  sidebar_width: 300,
  panel_layout: {
    fleet_visible: true,
    beads_visible: true,
    conversations_visible: true,
    patterns_visible: false,
  },
  filters: {
    bead_status: ['open', 'closed'],
    stitch_kind: ['operator', 'dictated', 'worker', 'ad-hoc'],
    conversation_filter: 'all',
    show_archived: false,
  },
  theme: 'auto',
});

// Read-write atom with server sync
export const uiStateAtom = atom(
  (get) => get(baseUiStateAtom),
  (get, set, newState: Partial<UiState>) => {
    const current = get(baseUiStateAtom);
    const updated = { ...current, ...newState };
    set(baseUiStateAtom, updated);
    // Trigger save to server
    saveUiStateToServer(updated);
  }
);

// Helper to parse JSON from server state
function parseServerState(stateJson: string): any {
  try {
    return JSON.parse(stateJson);
  } catch {
    return null;
  }
}

// Load UI state from server
async function loadUiStateFromServer(): Promise<UiState | null> {
  try {
    const response = await fetch('/api/ui/state');
    if (!response.ok) {
      console.warn('Failed to load UI state:', response.statusText);
      return null;
    }
    const data: UiStateResponse = await response.json();

    // Parse the state values from JSON strings
    const parsed: Partial<UiState> = {};
    for (const [key, value] of Object.entries(data.state)) {
      if (key === 'pinned_projects') {
        parsed.pinned_projects = parseServerState(value);
      } else if (key === 'active_project') {
        parsed.active_project = value === 'null' ? null : value;
      } else if (key === 'active_stitch') {
        parsed.active_stitch = value === 'null' ? null : value;
      } else if (key === 'sidebar_width') {
        parsed.sidebar_width = parseInt(value, 10);
      } else if (key === 'panel_layout') {
        parsed.panel_layout = parseServerState(value);
      } else if (key === 'filters') {
        parsed.filters = parseServerState(value);
      } else if (key === 'theme') {
        parsed.theme = parseServerState(value);
      }
    }

    return {
      schema_version: data.schema_version,
      pinned_projects: parsed.pinned_projects ?? [],
      active_project: parsed.active_project ?? null,
      active_stitch: parsed.active_stitch ?? null,
      sidebar_width: parsed.sidebar_width ?? 300,
      panel_layout: parsed.panel_layout ?? {
        fleet_visible: true,
        beads_visible: true,
        conversations_visible: true,
        patterns_visible: false,
      },
      filters: parsed.filters ?? {
        bead_status: ['open', 'closed'],
        stitch_kind: ['operator', 'dictated', 'worker', 'ad-hoc'],
        conversation_filter: 'all',
        show_archived: false,
      },
      theme: parsed.theme ?? 'auto',
    };
  } catch (e) {
    console.warn('Failed to load UI state from server:', e);
    return null;
  }
}

// Save UI state to server (debounced)
let saveTimeout: ReturnType<typeof setTimeout> | null = null;
function saveUiStateToServer(state: UiState) {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }

  saveTimeout = setTimeout(async () => {
    try {
      const stateMap: Record<string, string> = {};

      if (state.pinned_projects !== undefined) {
        stateMap.pinned_projects = JSON.stringify(state.pinned_projects);
      }
      if (state.active_project !== undefined) {
        stateMap.active_project = JSON.stringify(state.active_project);
      }
      if (state.active_stitch !== undefined) {
        stateMap.active_stitch = JSON.stringify(state.active_stitch);
      }
      if (state.sidebar_width !== undefined) {
        stateMap.sidebar_width = JSON.stringify(state.sidebar_width);
      }
      if (state.panel_layout !== undefined) {
        stateMap.panel_layout = JSON.stringify(state.panel_layout);
      }
      if (state.filters !== undefined) {
        stateMap.filters = JSON.stringify(state.filters);
      }
      if (state.theme !== undefined) {
        stateMap.theme = JSON.stringify(state.theme);
      }

      const response = await fetch('/api/ui/state/batch', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ state: stateMap }),
      });

      if (!response.ok) {
        console.warn('Failed to save UI state:', response.statusText);
      }
    } catch (e) {
      console.warn('Failed to save UI state to server:', e);
    }
  }, 500); // Debounce for 500ms
}

// Hook for UI state persistence
export function useUiState() {
  const [uiState, setUiState] = useAtom(uiStateAtom);
  const [isLoaded, setIsLoaded] = useState(false);
  const [operatorId, setOperatorId] = useState<string | null>(null);

  // Load state from server on mount
  useEffect(() => {
    let mounted = true;

    loadUiStateFromServer().then((serverState) => {
      if (!mounted) return;

      if (serverState) {
        setUiState(serverState);
      }
      setIsLoaded(true);

      // Extract operator ID from response
      loadUiStateFromServer().then((state) => {
        if (state && mounted) {
          // We'll get operator_id from a separate call if needed
        }
      });
    });

    return () => {
      mounted = false;
    };
  }, []);

  // Helper functions for common operations
  const setPinnedProjects = useCallback(
    (projects: string[]) => {
      setUiState({ pinned_projects: projects });
    },
    [setUiState]
  );

  const togglePinnedProject = useCallback(
    (project: string) => {
      const current = uiState.pinned_projects ?? [];
      const updated = current.includes(project)
        ? current.filter((p) => p !== project)
        : [...current, project];
      setUiState({ pinned_projects: updated });
    },
    [setUiState, uiState.pinned_projects]
  );

  const setActiveProject = useCallback(
    (project: string | null) => {
      setUiState({ active_project: project });
    },
    [setUiState]
  );

  const setActiveStitch = useCallback(
    (stitchId: string | null) => {
      setUiState({ active_stitch: stitchId });
    },
    [setUiState]
  );

  const setFilters = useCallback(
    (filters: Partial<UiStateFilters>) => {
      setUiState({ filters: { ...uiState.filters, ...filters } });
    },
    [setUiState, uiState.filters]
  );

  const setTheme = useCallback(
    (theme: 'light' | 'dark' | 'auto') => {
      setUiState({ theme });
    },
    [setUiState]
  );

  const setPanelLayout = useCallback(
    (layout: Partial<UiStatePanelLayout>) => {
      setUiState({ panel_layout: { ...uiState.panel_layout, ...layout } });
    },
    [setUiState, uiState.panel_layout]
  );

  const setSidebarWidth = useCallback(
    (width: number) => {
      setUiState({ sidebar_width: width });
    },
    [setUiState]
  );

  return {
    uiState,
    isLoaded,
    operatorId,
    setPinnedProjects,
    togglePinnedProject,
    setActiveProject,
    setActiveStitch,
    setFilters,
    setTheme,
    setPanelLayout,
    setSidebarWidth,
  };
}

// Export the base atom for direct use if needed
export { baseUiStateAtom };
