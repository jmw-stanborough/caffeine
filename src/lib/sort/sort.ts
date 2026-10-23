import type {
  SortDirection,
  SortDirectionLabels,
  SortFieldDefinition,
  SortRule,
} from "./types";

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

const STATUS_ERRORS_FIRST_RANK: Record<string, number> = {
  offline: 0,
  degraded: 1,
  checking: 2,
  online: 3,
};

const STATUS_HEALTHY_FIRST_RANK: Record<string, number> = {
  online: 0,
  checking: 1,
  degraded: 2,
  offline: 3,
};

export const STATUS_DIRECTION_LABELS: SortDirectionLabels = {
  desc: "Errors first (offline → degraded → online)",
  asc: "Healthy first (online → degraded → offline)",
};

export const TEXT_DIRECTION_LABELS: SortDirectionLabels = {
  asc: "Ascending (A → Z)",
  desc: "Descending (Z → A)",
};

export const NUMBER_DIRECTION_LABELS: SortDirectionLabels = {
  asc: "Lowest first (fastest)",
  desc: "Highest first (slowest)",
};

export function compareStatus(
  statusA: string | undefined,
  statusB: string | undefined,
  direction: SortDirection = "desc",
): number {
  const normA = (statusA ?? "checking").toLowerCase();
  const normB = (statusB ?? "checking").toLowerCase();

  const rankMap =
    direction === "desc"
      ? STATUS_ERRORS_FIRST_RANK
      : STATUS_HEALTHY_FIRST_RANK;

  const rankA = rankMap[normA] ?? 99;
  const rankB = rankMap[normB] ?? 99;

  return rankA - rankB;
}

export function compareNatural(
  a: unknown,
  b: unknown,
  direction: SortDirection = "asc",
): number {
  const hasA = a !== null && a !== undefined && a !== "";
  const hasB = b !== null && b !== undefined && b !== "";

  if (!hasA && !hasB) return 0;
  if (!hasA) return 1;
  if (!hasB) return -1;

  const strA = String(a);
  const strB = String(b);
  const diff = collator.compare(strA, strB);
  return direction === "asc" ? diff : -diff;
}

export function compareNumeric(
  a: unknown,
  b: unknown,
  direction: SortDirection = "asc",
): number {
  const hasA = typeof a === "number" && !Number.isNaN(a);
  const hasB = typeof b === "number" && !Number.isNaN(b);

  if (!hasA && !hasB) return 0;
  if (!hasA) return 1;
  if (!hasB) return -1;

  const diff = (a as number) - (b as number);
  return direction === "asc" ? diff : -diff;
}

export function sortEntities<T, TContext = unknown>(
  items: readonly T[],
  rules: readonly SortRule[],
  fields: readonly SortFieldDefinition<T, TContext>[],
  context?: TContext,
): T[] {
  if (rules.length === 0 || items.length <= 1) {
    return [...items];
  }

  const fieldMap = new Map<string, SortFieldDefinition<T, TContext>>();
  for (const f of fields) {
    fieldMap.set(f.id, f);
  }

  return [...items].sort((a, b) => {
    for (const rule of rules) {
      const field = fieldMap.get(rule.fieldId);
      if (!field) continue;

      let result = 0;
      if (field.compare) {
        result = field.compare(a, b, rule.direction, context);
      } else if (field.getValue) {
        const valA = field.getValue(a, context);
        const valB = field.getValue(b, context);
        result =
          typeof valA === "number" || typeof valB === "number"
            ? compareNumeric(valA, valB, rule.direction)
            : compareNatural(valA, valB, rule.direction);
      }

      if (result !== 0) {
        return result;
      }
    }
    return 0;
  });
}

export function loadSavedSortRules(storageKey: string): SortRule[] {
  if (typeof window === "undefined" || !window.localStorage) {
    return [];
  }
  const raw = window.localStorage.getItem(storageKey);
  if (!raw) return [];

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Failed to parse saved sort rules for ${storageKey}: ${String(err)}`);
  }

  if (!Array.isArray(parsed)) {
    throw new Error(`Invalid sort rules format in storage for ${storageKey}`);
  }

  return parsed.filter(
    (r): r is SortRule =>
      typeof r?.id === "string" &&
      typeof r?.fieldId === "string" &&
      (r?.direction === "asc" || r?.direction === "desc"),
  );
}

export function saveSortRules(storageKey: string, rules: SortRule[]): void {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }
  try {
    if (rules.length === 0) {
      window.localStorage.removeItem(storageKey);
    } else {
      window.localStorage.setItem(storageKey, JSON.stringify(rules));
    }
  } catch (err) {
    throw new Error(`Failed to save sort rules for ${storageKey}: ${String(err)}`);
  }
}
