import { useEffect, useMemo, useState } from "react";
import { Database } from "lucide-react";
import { api } from "./api";
import AppPage from "./components/AppPage";
import Sidebar from "./components/Sidebar";
import type { AppSummary, Country, Project } from "./types";

export default function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [projectId, setProjectId] = useState("");
  const [apps, setApps] = useState<AppSummary[]>([]);
  const [countries, setCountries] = useState<Country[]>([]);
  const [appId, setAppId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    Promise.all([api.projects(), api.countries()])
      .then(([items, countryItems]) => {
        setProjects(items);
        setCountries(countryItems);
        const stored = localStorage.getItem("asapi-project");
        const selected = items.find((project) => project.id === stored) ?? items[0];
        if (selected) setProjectId(selected.id);
      })
      .catch((reason: Error) => setError(reason.message))
      .finally(() => setLoading(false));
  }, []);

  const loadApps = async (nextProjectId: string, preferredAppId?: number) => {
    if (!nextProjectId) return;
    const items = await api.apps(nextProjectId);
    setApps(items);
    setAppId((current) => {
      const target = preferredAppId ?? current;
      return target != null && items.some((app) => app.apple_id === target)
        ? target
        : (items[0]?.apple_id ?? null);
    });
  };

  useEffect(() => {
    if (!projectId) return;
    localStorage.setItem("asapi-project", projectId);
    setLoading(true);
    setError("");
    loadApps(projectId)
      .catch((reason: Error) => setError(reason.message))
      .finally(() => setLoading(false));
  }, [projectId]);

  const createProject = async (name: string) => {
    const project = await api.createProject(name);
    setProjects((items) => [...items, project]);
    setProjectId(project.id);
  };

  const addApp = async (source: string, country: string) => {
    const match = source.trim().match(/(?:id)?(\d{6,})\D*$/);
    if (!match) {
      throw new Error("Enter a numeric App Store ID or an App Store URL.");
    }
    const appleId = Number(match[1]);
    await api.addApp(projectId, appleId, country);
    await loadApps(projectId, appleId);
  };

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
        onProjectChange={setProjectId}
        apps={apps}
        countries={countries}
        selectedAppId={appId}
        onAppChange={setAppId}
        onCreateProject={createProject}
        onAddApp={addApp}
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
            onAppChanged={() => loadApps(projectId, selectedApp.apple_id)}
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
