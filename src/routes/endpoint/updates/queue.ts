export function normalizeTargetIds(target?: string | string[]): {
  single: string | null;
  list: string[] | null;
} {
  if (!target) {
    return { single: null, list: null };
  }
  if (Array.isArray(target)) {
    return {
      single: target.length === 1 ? target[0] : null,
      list: target,
    };
  }
  return { single: target, list: [target] };
}
