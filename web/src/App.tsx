import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Database } from "lucide-react";
import { api } from "./api";
import AppPage from "./components/AppPage";
import AppSearchDialog from "./components/AppSearchDialog";
import Sidebar from "./components/Sidebar";
import type { AppSummary, Country, Project } from "./types";

interface WorkspaceRoute {
  projectId: string | null;
  appId: number | null;
}

function readWorkspaceRoute(): WorkspaceRoute {
  const match = window.location.pathname.match(
    /^\/projects\/([^/]+)(?:\/apps\/(\d+))?\/?$/,
  );
  if (!match) return { projectId: null, appId: null };
  try {
    const appId = match[2] ? Number(match[2]) : null;
    return {
      projectId: decodeURIComponent(match[1]),
      appId: appId != null && Number.isSafeInteger(appId) ? appId : null,
    };
  } catch {
    return { projectId: null, appId: null };
  }
}

function workspacePath(projectId: string, appId: number | null) {
  const projectPath = `/projects/${encodeURIComponent(projectId)}`;
  return appId == null ? projectPath : `${projectPath}/apps/${appId}`;
}

function writeWorkspaceRoute(
  projectId: string,
  appId: number | null,
  mode: "push" | "replace",
) {
  const path = workspacePath(projectId, appId);
  if (window.location.pathname === path) return;
  window.history[mode === "push" ? "pushState" : "replaceState"]({}, "", path);
}

export default function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [projectId, setProjectId] = useState("");
  const [apps, setApps] = useState<AppSummary[]>([]);
  const [countries, setCountries] = useState<Country[]>([]);
  const [appId, setAppId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [showAppSearch, setShowAppSearch] = useState(false);
  const activeProjectId = useRef("");
  const appRequestVersions = useRef(new Map<string, number>());

  useEffect(() => {
    Promise.all([api.projects(), api.countries()])
      .then(([items, countryItems]) => {
        setProjects(items);
        setCountries(countryItems);
        const route = readWorkspaceRoute();
        const stored = localStorage.getItem("asapi-project");
        const selected =
          items.find((project) => project.id === route.projectId) ??
          items.find((project) => project.id === stored) ??
          items[0];
        if (selected) {
          activeProjectId.current = selected.id;
          setProjectId(selected.id);
          setAppId(route.projectId === selected.id ? route.appId : null);
        }
      })
      .catch((reason: Error) => setError(reason.message))
      .finally(() => setLoading(false));
  }, []);

  const loadApps = useCallback(
    async (nextProjectId: string, preferredAppId?: number) => {
      if (!nextProjectId) return null;
      const requestVersion =
        (appRequestVersions.current.get(nextProjectId) ?? 0) + 1;
      appRequestVersions.current.set(nextProjectId, requestVersion);
      const items = await api.apps(nextProjectId);
      if (
        activeProjectId.current !== nextProjectId ||
        appRequestVersions.current.get(nextProjectId) !== requestVersion
      ) {
        return null;
      }
      setApps(items);
      setAppId((current) => {
        const target = preferredAppId ?? current;
        return target != null && items.some((app) => app.apple_id === target)
          ? target
          : (items[0]?.apple_id ?? null);
      });
      return items;
    },
    [],
  );

  useEffect(() => {
    if (!projectId) return;
    localStorage.setItem("asapi-project", projectId);
    setLoading(true);
    setError("");
    loadApps(projectId)
      .catch((reason: Error) => setError(reason.message))
      .finally(() => {
        if (activeProjectId.current === projectId) setLoading(false);
      });
  }, [loadApps, projectId]);

  useEffect(() => {
    if (projectId) writeWorkspaceRoute(projectId, appId, "replace");
  }, [projectId, appId]);

  useEffect(() => {
    if (!projects.length) return;
    const handlePopState = () => {
      const route = readWorkspaceRoute();
      const nextProject =
        projects.find((project) => project.id === route.projectId) ??
        projects[0];
      if (!nextProject) return;
      const projectChanged = activeProjectId.current !== nextProject.id;
      activeProjectId.current = nextProject.id;
      if (projectChanged) setApps([]);
      setProjectId(nextProject.id);
      setAppId(route.projectId === nextProject.id ? route.appId : null);
    };
    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, [projects]);

  const changeProject = useCallback((nextProjectId: string) => {
    if (activeProjectId.current === nextProjectId) return;
    activeProjectId.current = nextProjectId;
    setApps([]);
    setAppId(null);
    setProjectId(nextProjectId);
    writeWorkspaceRoute(nextProjectId, null, "push");
  }, []);

  const changeApp = useCallback(
    (nextAppId: number) => {
      setAppId(nextAppId);
      writeWorkspaceRoute(projectId, nextAppId, "push");
    },
    [projectId],
  );

  const createProject = async (name: string) => {
    const project = await api.createProject(name);
    setProjects((items) => [...items, project]);
    changeProject(project.id);
  };

  const addApp = async (source: string, country: string) => {
    const match = source.trim().match(/(?:id)?(\d{6,})\D*$/);
    if (!match) {
      throw new Error("Enter a numeric App Store ID or an App Store URL.");
    }
    const appleId = Number(match[1]);
    await api.addApp(projectId, appleId, country);
    const items = await loadApps(projectId, appleId);
    if (
      items?.some((app) => app.apple_id === appleId) &&
      activeProjectId.current === projectId
    ) {
      writeWorkspaceRoute(projectId, appleId, "push");
    }
  };

  const reloadCurrentApps = useCallback(async () => {
    await loadApps(projectId);
  }, [loadApps, projectId]);

  const selectedApp = useMemo(
    () => apps.find((app) => app.apple_id === appId) ?? null,
    [apps, appId],
  );
  const selectedProject = projects.find((project) => project.id === projectId);

  return (
    <div className="shell">
      <Sidebar
        projects={projects}
        projectId={projectId}
        onProjectChange={changeProject}
        apps={apps}
        countries={countries}
        selectedAppId={appId}
        onAppChange={changeApp}
        onCreateProject={createProject}
        onAddApp={addApp}
        onOpenSearch={() => setShowAppSearch(true)}
      />
      <main className="workspace">
        {error ? <div className="global-error">{error}</div> : null}
        {loading && !selectedApp ? (
          <div className="center-state">
            <span className="registry-loader" aria-hidden="true">
              <i />
              <i />
              <i />
            </span>
            <p>Loading workspace</p>
          </div>
        ) : selectedApp && selectedProject ? (
          <AppPage
            key={`${projectId}:${selectedApp.apple_id}`}
            project={selectedProject}
            app={selectedApp}
            countries={countries}
            onAppChanged={reloadCurrentApps}
          />
        ) : selectedProject ? (
          <EmptyProject project={selectedProject} />
        ) : (
          <div className="center-state">
            <Database size={25} />
            <p>No project is available.</p>
          </div>
        )}
      </main>
      {showAppSearch ? (
        <AppSearchDialog
          countries={countries}
          onClose={() => setShowAppSearch(false)}
        />
      ) : null}
    </div>
  );
}

function EmptyProject({ project }: { project: Project }) {
  return (
    <div className="empty-project">
      <h1>Start collecting App Store research.</h1>
      <p>
        Add an app from the Apps control in the sidebar. Use a numeric App Store
        ID or paste its App Store URL; the US storefront is selected by default.
      </p>
      <div className="project-stamp">Project: {project.name}</div>
    </div>
  );
}
