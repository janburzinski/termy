import { createFileRoute } from "@tanstack/react-router";
import type { FormEvent, JSX } from "react";
import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export const Route = createFileRoute("/themes")({
  component: ThemesPage,
});

interface Theme {
  id: string;
  name: string;
  slug: string;
  description: string;
  latestVersion: string | null;
  fileKey: string | null;
  githubUsernameClaim: string;
  githubUserIdClaim: number | null;
  isPublic: boolean;
  createdAt: string;
  updatedAt: string;
}

interface ThemeVersion {
  id: string;
  themeId: string;
  version: string;
  fileKey: string;
  changelog: string;
  checksumSha256: string | null;
  createdBy: string | null;
  publishedAt: string;
  createdAt: string;
}

interface AuthUser {
  id: string;
  githubUserId: number;
  githubLogin: string;
}

interface ThemeWithVersionsResponse {
  theme: Theme;
  versions: ThemeVersion[];
}

interface ApiErrorBody {
  error?: string;
}

function ThemesPage(): JSX.Element {
  const apiBase = useMemo(() => "/theme-api", []);

  const [user, setUser] = useState<AuthUser | null>(null);
  const [themes, setThemes] = useState<Theme[]>([]);
  const [selectedSlug, setSelectedSlug] = useState<string>("");
  const [selectedVersions, setSelectedVersions] = useState<ThemeVersion[]>([]);
  const [createName, setCreateName] = useState("");
  const [createSlug, setCreateSlug] = useState("");
  const [createDescription, setCreateDescription] = useState("");
  const [createIsPublic, setCreateIsPublic] = useState(true);
  const [updateName, setUpdateName] = useState("");
  const [updateDescription, setUpdateDescription] = useState("");
  const [updateIsPublic, setUpdateIsPublic] = useState(true);
  const [publishVersion, setPublishVersion] = useState("");
  const [publishFileKey, setPublishFileKey] = useState("");
  const [publishChangelog, setPublishChangelog] = useState("");
  const [publishChecksum, setPublishChecksum] = useState("");
  const [isBootstrapping, setIsBootstrapping] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const selectedTheme =
    themes.find((theme) => theme.slug === selectedSlug) ?? null;
  const canEditSelectedTheme =
    Boolean(user) &&
    Boolean(selectedTheme) &&
    selectedTheme?.githubUsernameClaim.toLowerCase() ===
      user?.githubLogin.toLowerCase();

  const loginUrl = useMemo(() => {
    if (typeof window === "undefined") {
      return `${apiBase}/auth/github/login`;
    }
    const redirectTo = `${window.location.origin}/themes`;
    return `${apiBase}/auth/github/login?redirect_to=${encodeURIComponent(redirectTo)}`;
  }, [apiBase]);

  useEffect(() => {
    void bootstrap();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!selectedTheme) {
      return;
    }

    setUpdateName(selectedTheme.name);
    setUpdateDescription(selectedTheme.description);
    setUpdateIsPublic(selectedTheme.isPublic);
    setPublishFileKey(selectedTheme.fileKey ?? "");
  }, [selectedTheme]);

  async function bootstrap(): Promise<void> {
    try {
      setError(null);
      setIsBootstrapping(true);
      const [currentUser, loadedThemes] = await Promise.all([
        fetchCurrentUser(),
        fetchThemes(),
      ]);

      setUser(currentUser);
      setThemes(loadedThemes);

      if (loadedThemes.length > 0) {
        const firstSlug = loadedThemes[0].slug;
        setSelectedSlug(firstSlug);
        await loadThemeVersions(firstSlug);
      }
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsBootstrapping(false);
    }
  }

  async function request<T>(path: string, init?: RequestInit): Promise<T> {
    const response = await fetch(`${apiBase}${path}`, {
      ...init,
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        ...(init?.headers ?? {}),
      },
    });

    if (!response.ok) {
      let message = `Request failed (${response.status})`;
      try {
        const body = (await response.json()) as ApiErrorBody;
        if (body.error) {
          message = body.error;
        }
      } catch {
        // keep default message
      }
      throw new Error(message);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return (await response.json()) as T;
  }

  async function fetchCurrentUser(): Promise<AuthUser | null> {
    const response = await fetch(`${apiBase}/auth/me`, {
      credentials: "include",
    });

    if (response.status === 401) {
      return null;
    }

    if (!response.ok) {
      let message = `Could not resolve session (${response.status})`;
      try {
        const body = (await response.json()) as ApiErrorBody;
        if (body.error) {
          message = body.error;
        }
      } catch {
        // keep default message
      }
      throw new Error(message);
    }

    return (await response.json()) as AuthUser;
  }

  async function fetchThemes(): Promise<Theme[]> {
    return request<Theme[]>("/themes", { method: "GET" });
  }

  async function loadThemeVersions(slug: string): Promise<void> {
    try {
      const response = await request<ThemeWithVersionsResponse>(
        `/themes/${slug}/versions`,
        {
          method: "GET",
        },
      );
      setSelectedVersions(response.versions);
    } catch (err) {
      setSelectedVersions([]);
      setError(getErrorMessage(err));
    }
  }

  async function handleLogout(): Promise<void> {
    try {
      setError(null);
      await request<void>("/auth/logout", { method: "POST" });
      setUser(null);
      setNotice("Logged out.");
    } catch (err) {
      setError(getErrorMessage(err));
    }
  }

  async function handleCreateTheme(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();
    try {
      setError(null);
      setNotice(null);
      setIsSubmitting(true);

      const created = await request<Theme>("/themes", {
        method: "POST",
        body: JSON.stringify({
          name: createName,
          slug: createSlug,
          description: createDescription,
          isPublic: createIsPublic,
        }),
      });

      setThemes((prev) => [
        created,
        ...prev.filter((item) => item.id !== created.id),
      ]);
      setSelectedSlug(created.slug);
      setSelectedVersions([]);
      setCreateName("");
      setCreateSlug("");
      setCreateDescription("");
      setNotice(`Theme '${created.slug}' created.`);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handleUpdateTheme(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();
    if (!selectedTheme) {
      return;
    }

    try {
      setError(null);
      setNotice(null);
      setIsSubmitting(true);

      const updated = await request<Theme>(`/themes/${selectedTheme.slug}`, {
        method: "PATCH",
        body: JSON.stringify({
          name: updateName,
          description: updateDescription,
          isPublic: updateIsPublic,
        }),
      });

      setThemes((prev) =>
        prev.map((item) => (item.id === updated.id ? updated : item)),
      );
      setNotice(`Theme '${updated.slug}' updated.`);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handlePublishVersion(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();
    if (!selectedTheme) {
      return;
    }

    try {
      setError(null);
      setNotice(null);
      setIsSubmitting(true);

      await request<{ theme: Theme; version: ThemeVersion }>(
        `/themes/${selectedTheme.slug}/versions`,
        {
          method: "POST",
          body: JSON.stringify({
            version: publishVersion,
            fileKey: publishFileKey,
            changelog: publishChangelog,
            checksumSha256: publishChecksum || undefined,
          }),
        },
      );

      const loadedThemes = await fetchThemes();
      setThemes(loadedThemes);
      await loadThemeVersions(selectedTheme.slug);
      setPublishVersion("");
      setPublishChangelog("");
      setPublishChecksum("");
      setNotice("Version published.");
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handleSelectTheme(slug: string): Promise<void> {
    setSelectedSlug(slug);
    setNotice(null);
    setError(null);
    await loadThemeVersions(slug);
  }

  return (
    <section className="pt-28 pb-16">
      <div className="mx-auto max-w-6xl space-y-6">
        <div className="rounded-3xl border border-border/50 bg-gradient-to-br from-card via-card to-secondary/50 p-6 md:p-8">
          <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">
            Theme Store
          </p>
          <h1 className="mt-3 text-3xl font-semibold md:text-5xl">
            Upload and manage Termy themes
          </h1>
          <p className="mt-3 max-w-2xl text-muted-foreground">
            Sign in with GitHub, create themes, and publish new versions to your
            catalog.
          </p>
          <div className="mt-5 flex flex-wrap items-center gap-3">
            {user ? (
              <>
                <div className="rounded-lg border border-border/60 bg-background/80 px-3 py-2 text-sm">
                  Signed in as{" "}
                  <span className="font-medium">@{user.githubLogin}</span>
                </div>
                <Button type="button" variant="outline" onClick={handleLogout}>
                  Log out
                </Button>
              </>
            ) : (
              <a href={loginUrl}>
                <Button type="button">Login with GitHub</Button>
              </a>
            )}
          </div>
        </div>

        {error && (
          <div className="rounded-xl border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive">
            {error}
          </div>
        )}

        {notice && (
          <div className="rounded-xl border border-primary/40 bg-primary/10 px-4 py-3 text-sm text-foreground">
            {notice}
          </div>
        )}

        <div className="grid gap-6 lg:grid-cols-[360px_minmax(0,1fr)]">
          <Card className="border-border/60">
            <CardHeader>
              <CardTitle>Available Themes</CardTitle>
              <CardDescription>
                {isBootstrapping
                  ? "Loading themes..."
                  : `${themes.length} themes available`}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {themes.map((theme) => (
                <button
                  key={theme.id}
                  type="button"
                  className={`w-full rounded-lg border px-3 py-2 text-left transition-colors ${
                    selectedSlug === theme.slug
                      ? "border-primary/60 bg-primary/10"
                      : "border-border/50 bg-background hover:border-primary/30"
                  }`}
                  onClick={() => void handleSelectTheme(theme.slug)}
                >
                  <div className="flex items-center justify-between gap-3">
                    <p className="font-medium">{theme.name}</p>
                    <span className="text-xs text-muted-foreground">
                      {theme.latestVersion ?? "no versions"}
                    </span>
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    /{theme.slug}
                  </p>
                </button>
              ))}
              {!isBootstrapping && themes.length === 0 && (
                <p className="text-sm text-muted-foreground">No themes yet.</p>
              )}
            </CardContent>
          </Card>

          <div className="space-y-6">
            <Card className="border-border/60">
              <CardHeader>
                <CardTitle>Create Theme</CardTitle>
                <CardDescription>
                  Creates a new theme owned by your authenticated GitHub
                  account.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <form
                  className="space-y-3"
                  onSubmit={(event) => void handleCreateTheme(event)}
                >
                  <input
                    className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                    placeholder="Theme name"
                    value={createName}
                    onChange={(event) => setCreateName(event.target.value)}
                    disabled={!user || isSubmitting}
                  />
                  <input
                    className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                    placeholder="slug (e.g. nord-night)"
                    value={createSlug}
                    onChange={(event) =>
                      setCreateSlug(event.target.value.toLowerCase())
                    }
                    disabled={!user || isSubmitting}
                  />
                  <textarea
                    className="min-h-20 w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                    placeholder="Description"
                    value={createDescription}
                    onChange={(event) =>
                      setCreateDescription(event.target.value)
                    }
                    disabled={!user || isSubmitting}
                  />
                  <label className="flex items-center gap-2 text-sm text-muted-foreground">
                    <input
                      type="checkbox"
                      checked={createIsPublic}
                      onChange={(event) =>
                        setCreateIsPublic(event.target.checked)
                      }
                      disabled={!user || isSubmitting}
                    />
                    Public theme
                  </label>
                  <Button type="submit" disabled={!user || isSubmitting}>
                    Create theme
                  </Button>
                </form>
              </CardContent>
            </Card>

            <Card className="border-border/60">
              <CardHeader>
                <CardTitle>Selected Theme</CardTitle>
                <CardDescription>
                  {selectedTheme
                    ? `${selectedTheme.name} (${selectedTheme.slug})`
                    : "Select a theme from the list"}
                </CardDescription>
              </CardHeader>
              <CardContent>
                {!selectedTheme && (
                  <p className="text-sm text-muted-foreground">
                    No theme selected.
                  </p>
                )}
                {selectedTheme && (
                  <div className="space-y-6">
                    <form
                      className="space-y-3"
                      onSubmit={(event) => void handleUpdateTheme(event)}
                    >
                      <h3 className="text-sm font-semibold">Update metadata</h3>
                      <input
                        className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                        value={updateName}
                        onChange={(event) => setUpdateName(event.target.value)}
                        disabled={!canEditSelectedTheme || isSubmitting}
                      />
                      <textarea
                        className="min-h-20 w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                        value={updateDescription}
                        onChange={(event) =>
                          setUpdateDescription(event.target.value)
                        }
                        disabled={!canEditSelectedTheme || isSubmitting}
                      />
                      <label className="flex items-center gap-2 text-sm text-muted-foreground">
                        <input
                          type="checkbox"
                          checked={updateIsPublic}
                          onChange={(event) =>
                            setUpdateIsPublic(event.target.checked)
                          }
                          disabled={!canEditSelectedTheme || isSubmitting}
                        />
                        Public theme
                      </label>
                      <Button
                        type="submit"
                        disabled={!canEditSelectedTheme || isSubmitting}
                      >
                        Save changes
                      </Button>
                    </form>

                    <form
                      className="space-y-3"
                      onSubmit={(event) => void handlePublishVersion(event)}
                    >
                      <h3 className="text-sm font-semibold">
                        Publish new version
                      </h3>
                      <input
                        className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                        placeholder="Version (e.g. 1.2.0)"
                        value={publishVersion}
                        onChange={(event) =>
                          setPublishVersion(event.target.value)
                        }
                        disabled={!canEditSelectedTheme || isSubmitting}
                      />
                      <input
                        className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                        placeholder="S3 file key"
                        value={publishFileKey}
                        onChange={(event) =>
                          setPublishFileKey(event.target.value)
                        }
                        disabled={!canEditSelectedTheme || isSubmitting}
                      />
                      <textarea
                        className="min-h-20 w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                        placeholder="Changelog"
                        value={publishChangelog}
                        onChange={(event) =>
                          setPublishChangelog(event.target.value)
                        }
                        disabled={!canEditSelectedTheme || isSubmitting}
                      />
                      <input
                        className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                        placeholder="Checksum SHA256 (optional)"
                        value={publishChecksum}
                        onChange={(event) =>
                          setPublishChecksum(event.target.value)
                        }
                        disabled={!canEditSelectedTheme || isSubmitting}
                      />
                      <Button
                        type="submit"
                        disabled={!canEditSelectedTheme || isSubmitting}
                      >
                        Publish version
                      </Button>
                    </form>

                    <div>
                      <h3 className="text-sm font-semibold mb-2">
                        Version history
                      </h3>
                      <div className="space-y-2">
                        {selectedVersions.map((version) => (
                          <div
                            key={version.id}
                            className="rounded-lg border border-border/60 px-3 py-2"
                          >
                            <div className="flex items-center justify-between gap-3">
                              <span className="font-medium">
                                {version.version}
                              </span>
                              <span className="text-xs text-muted-foreground">
                                {new Date(version.publishedAt).toLocaleString()}
                              </span>
                            </div>
                            <p className="mt-1 text-xs text-muted-foreground">
                              {version.fileKey}
                            </p>
                            {version.changelog && (
                              <p className="mt-2 text-sm text-muted-foreground">
                                {version.changelog}
                              </p>
                            )}
                          </div>
                        ))}
                        {selectedVersions.length === 0 && (
                          <p className="text-sm text-muted-foreground">
                            No versions published yet.
                          </p>
                        )}
                      </div>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </section>
  );
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return "Unexpected error";
}
