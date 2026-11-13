export interface Breadcrumb {
  label: string;
  fullPath: string;
}

const WINDOWS_DRIVE_PREFIX = /^[a-zA-Z]:/;
const WINDOWS_DRIVE_ONLY = /^[a-zA-Z]:$/;

export function buildBreadcrumbs(currentPath: string): Breadcrumb[] {
  if (!currentPath) return [];

  if (WINDOWS_DRIVE_PREFIX.test(currentPath)) {
    const parts = currentPath.split(/[/\\]+/).filter(Boolean);
    if (parts.length === 0) return [];

    const root = `${parts[0]}\\`;
    const crumbs: Breadcrumb[] = [{ label: parts[0], fullPath: root }];
    let accumulator = root;

    for (let i = 1; i < parts.length; i++) {
      accumulator = `${accumulator}${accumulator.endsWith("\\") ? "" : "\\"}${parts[i]}`;
      crumbs.push({ label: parts[i], fullPath: accumulator });
    }
    return crumbs;
  }

  const parts = currentPath.split("/").filter(Boolean);
  if (currentPath.startsWith("/")) {
    const crumbs: Breadcrumb[] = [{ label: "/", fullPath: "/" }];
    let accumulator = "";
    for (const part of parts) {
      accumulator += `/${part}`;
      crumbs.push({ label: part, fullPath: accumulator });
    }
    return crumbs;
  }

  const crumbs: Breadcrumb[] = [];
  let accumulator = "";
  for (const part of parts) {
    accumulator = accumulator ? `${accumulator}/${part}` : part;
    crumbs.push({ label: part, fullPath: accumulator });
  }
  return crumbs;
}

export function getParentPath(currentPath: string): string | null {
  if (!currentPath) return null;

  if (WINDOWS_DRIVE_PREFIX.test(currentPath)) {
    const trimmed = currentPath.replace(/[/\\]+$/, "");
    if (WINDOWS_DRIVE_ONLY.test(trimmed)) {
      return null;
    }

    const lastSlash = Math.max(trimmed.lastIndexOf("/"), trimmed.lastIndexOf("\\"));
    if (lastSlash < 0) return null;

    const parent = trimmed.slice(0, lastSlash);
    if (WINDOWS_DRIVE_ONLY.test(parent)) {
      // Append backslash so Windows does not treat "C:" as CWD-relative.
      return `${parent}\\`;
    }
    return parent;
  }

  const trimmed = currentPath.replace(/\/+$/, "");
  if (trimmed === "") return null;

  const lastSlash = trimmed.lastIndexOf("/");
  if (lastSlash < 0) return null;

  return trimmed.slice(0, lastSlash) || "/";
}
