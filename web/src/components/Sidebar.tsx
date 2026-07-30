import { Blocks, Plus, Star, X } from "lucide-react";
import { useMemo, useState } from "react";
import { countryFlag, formatCount } from "../format";
import type { AppSummary, Country, Project } from "../types";
import Picker from "./Picker";

interface Props {
  projects: Project[];
  projectId: string;
  onProjectChange: (id: string) => void;
  apps: AppSummary[];
  countries: Country[];
  selectedAppId: number | null;
  onAppChange: (id: number) => void;
  onCreateProject: (name: string) => Promise<void>;
  onAddApp: (source: string, country: string) => Promise<void>;
}

export default function Sidebar({
  projects,
  projectId,
  onProjectChange,
  apps,
  countries,
  selectedAppId,
  onAppChange,
  onCreateProject,
  onAddApp,
}: Props) {
  const [projectForm, setProjectForm] = useState(false);
  const [projectName, setProjectName] = useState("");
  const [appForm, setAppForm] = useState(false);
  const [appSource, setAppSource] = useState("");
  const [appCountry, setAppCountry] = useState("us");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const sortedApps = useMemo(
    () =>
      [...apps].sort((left, right) => {
        const ratingDifference =
          (right.rating_count ?? -1) - (left.rating_count ?? -1);
        if (ratingDifference) return ratingDifference;
        return (left.name ?? String(left.apple_id)).localeCompare(
          right.name ?? String(right.apple_id),
          undefined,
          { sensitivity: "base" },
        );
      }),
    [apps],
  );

  const submitProject = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!projectName.trim()) return;
    setBusy(true);
    setError("");
    try {
      await onCreateProject(projectName.trim());
      setProjectName("");
      setProjectForm(false);
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const submitApp = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!appSource.trim()) return;
    setBusy(true);
    setError("");
    try {
      await onAddApp(appSource, appCountry);
      setAppSource("");
      setAppForm(false);
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <aside className="sidebar">
      <div className="brand">
        <div>
          <strong>asapi</strong>
          <span>local research registry</span>
        </div>
      </div>

      <div className="project-select">
        <span>
          Project
          <button
            className="line-action"
            type="button"
            title={projectForm ? "Close project form" : "Add project"}
            onClick={() => setProjectForm((visible) => !visible)}
          >
            {projectForm ? <X size={12} /> : <Plus size={12} />}
          </button>
        </span>
        <Picker
          value={projectId}
          options={projects.map((project) => ({
            value: project.id,
            label: project.name,
            meta: project.id.slice(0, 8),
            icon: <Blocks size={13} />,
          }))}
          onChange={onProjectChange}
          ariaLabel="Project"
          className="project-picker"
          searchPlaceholder="Search projects"
        />
        {projectForm ? (
          <form className="sidebar-form" onSubmit={submitProject}>
            <input
              value={projectName}
              onChange={(event) => setProjectName(event.target.value)}
              placeholder="Project name"
              maxLength={80}
              autoFocus
            />
            <button type="submit" disabled={busy || !projectName.trim()}>
              Create
            </button>
          </form>
        ) : null}
      </div>

      <div className="sidebar-section-title">
        <span>Apps</span>
        <span className="sidebar-title-actions">
          <small>{apps.length}</small>
          <button
            className="line-action"
            type="button"
            title={appForm ? "Close app form" : "Add app"}
            onClick={() => setAppForm((visible) => !visible)}
          >
            {appForm ? <X size={12} /> : <Plus size={12} />}
          </button>
        </span>
      </div>
      {appForm ? (
        <form className="sidebar-form app-create-form" onSubmit={submitApp}>
          <input
            value={appSource}
            onChange={(event) => setAppSource(event.target.value)}
            placeholder="App ID or App Store URL"
            autoFocus
          />
          <div>
            <Picker
              value={appCountry}
              options={countries.map((country) => ({
                value: country.code,
                label: country.name,
                triggerLabel: country.code.toUpperCase(),
                meta: country.code.toUpperCase(),
                icon: countryFlag(country.code),
              }))}
              onChange={setAppCountry}
              ariaLabel="Main storefront"
              className="sidebar-country-picker"
              searchPlaceholder="Search countries"
            />
            <button type="submit" disabled={busy || !appSource.trim()}>
              Add
            </button>
          </div>
        </form>
      ) : null}
      {error ? <div className="sidebar-form-error">{error}</div> : null}
      <nav className="app-list">
        {sortedApps.map((app) => (
          <button
            className={selectedAppId === app.apple_id ? "active" : ""}
            key={app.apple_id}
            onClick={() => onAppChange(app.apple_id)}
          >
            {app.icon_url ? (
              <img src={app.icon_url} alt="" />
            ) : (
              <span className="app-placeholder">
                {(app.name ?? String(app.apple_id)).slice(0, 1)}
              </span>
            )}
            <span className="app-list-copy">
              <strong>{app.name ?? app.apple_id}</strong>
              <small>
                {countryFlag(app.main_country)}{" "}
                {app.rating != null ? (
                  <>
                    <Star size={10} fill="currentColor" /> {app.rating.toFixed(1)}
                    <span className="dot">·</span>
                  </>
                ) : null}
                {formatCount(app.rating_count)}
              </small>
            </span>
          </button>
        ))}
      </nav>

      <div className="sidebar-footer">
        Local server
        <a href="/api/openapi.json" target="_blank" rel="noreferrer">
          API
        </a>
      </div>
    </aside>
  );
}
