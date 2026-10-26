export function isDashboardFeature(item: {
  id?: string;
  href?: string;
  label?: string;
}): boolean {
  if (item.href === "/") return true;
  if (item.id?.trim().toLowerCase() === "dashboard") return true;
  if (item.label?.trim().toLowerCase() === "dashboard") return true;
  return false;
}

export function compareFeaturesAlphabetically<T extends { label: string }>(
  a: T,
  b: T,
): number {
  return a.label.localeCompare(b.label, undefined, {
    sensitivity: "base",
    numeric: true,
  });
}

export function orderSidebarItems<
  T extends { href?: string; id?: string; label: string },
>(items: readonly T[]): T[] {
  const dashboardItem = items.find(isDashboardFeature);
  const featureItems = items
    .filter((item) => item !== dashboardItem && !isDashboardFeature(item))
    .slice()
    .sort(compareFeaturesAlphabetically);

  return dashboardItem ? [dashboardItem, ...featureItems] : featureItems;
}

export function orderDashboardWidgets<
  T extends { href?: string; id?: string; label: string },
>(widgets: readonly T[]): T[] {
  return widgets
    .filter((widget) => !isDashboardFeature(widget))
    .slice()
    .sort(compareFeaturesAlphabetically);
}

export function orderFeatures<
  T extends { href?: string; id?: string; label: string },
>(features: readonly T[]): T[] {
  return features
    .filter((feature) => !isDashboardFeature(feature))
    .slice()
    .sort(compareFeaturesAlphabetically);
}
