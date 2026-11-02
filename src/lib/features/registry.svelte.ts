import {
  Cpu,
  DesktopPc,
  HardDrive,
  LayoutDashboard,
  Network,
  Printer,
  Server,
  Terminal,
  Updates,
} from "$lib/components/icons";
import EndpointSummaryCard from "$lib/components/endpoint-summary-card.svelte";
import PrinterSummaryCard from "$lib/components/printer-summary-card.svelte";
import ServerSummaryCard from "$lib/components/server-summary-card.svelte";
import TerminalWorkspace from "../../routes/terminal/terminal-workspace.svelte";
import { printerStore } from "../../routes/printers/printers.svelte";
import { serverStore } from "../../routes/servers/servers.svelte";
import {
  orderDashboardWidgets,
  orderFeatures,
  orderSidebarItems,
  orderSidebarSections,
} from "./order";
import type {
  DashboardWidget,
  FeatureDefinition,
  PersistentFeatureView,
  SidebarItem,
  SidebarSection,
} from "./types";

export const DASHBOARD_NAV_ITEM: SidebarItem = {
  href: "/",
  label: "Dashboard",
  icon: LayoutDashboard,
};

export const SIDEBAR_SUBHEADERS: Record<
  string,
  { label: string; icon: FeatureDefinition["icon"] }
> = {
  network: {
    label: "Network",
    icon: Network,
  },
  endpoint: {
    label: "Endpoint",
    icon: DesktopPc,
  },
};

const registeredFeatures = $state<FeatureDefinition[]>([
  {
    id: "hardware",
    label: "Hardware",
    href: "/hardware",
    icon: Cpu,
    widget: EndpointSummaryCard,
    group: "endpoint",
  },
  {
    id: "filesystem",
    label: "File System",
    href: "/filesystem",
    icon: HardDrive,
    group: "endpoint",
  },
  {
    id: "terminal",
    label: "Terminal",
    href: "/terminal",
    icon: Terminal,
    group: "endpoint",
    backgroundLoadable: true,
    persistentComponent: TerminalWorkspace,
  },
  {
    id: "updates",
    label: "Updates",
    href: "/updates",
    icon: Updates,
    group: "endpoint",
  },
  {
    id: "printers",
    label: "Printers",
    href: "/printers",
    icon: Printer,
    badgeCount: () => printerStore.alertCount,
    widget: PrinterSummaryCard,
    group: "network",
    backgroundLoadable: true,
    init: () => printerStore.init(),
    onSnapshotSync: () => printerStore.fetchPrinters(),
  },
  {
    id: "servers",
    label: "Servers",
    href: "/servers",
    icon: Server,
    badgeCount: () => serverStore.alertCount,
    widget: ServerSummaryCard,
    group: "network",
    backgroundLoadable: true,
    init: () => serverStore.init(),
    onSnapshotSync: () => serverStore.fetchServers(),
  },
]);

export function registerFeature(feature: FeatureDefinition): void {
  const existingIndex = registeredFeatures.findIndex((f) => f.id === feature.id);
  if (existingIndex >= 0) {
    registeredFeatures[existingIndex] = feature;
  } else {
    registeredFeatures.push(feature);
  }
}

export function getRegisteredFeatures(): FeatureDefinition[] {
  return orderFeatures(registeredFeatures);
}

export function registerSidebarSubheader(
  groupId: string,
  meta: { label: string; icon: FeatureDefinition["icon"] },
): void {
  SIDEBAR_SUBHEADERS[groupId] = meta;
}

export function getSidebarSections(): SidebarSection[] {
  const features = getRegisteredFeatures();

  for (const feature of features) {
    if (feature.group && !SIDEBAR_SUBHEADERS[feature.group]) {
      throw new Error(
        `Missing sidebar subheader definition for group "${feature.group}" in feature "${feature.id}"`,
      );
    }
  }

  const sections: SidebarSection[] = Object.entries(SIDEBAR_SUBHEADERS).map(
    ([groupId, meta]) => {
      const items: SidebarItem[] = features
        .filter((f) => f.group === groupId)
        .map((feature) => ({
          href: feature.href,
          label: feature.label,
          icon: feature.icon,
          badgeCount:
            typeof feature.badgeCount === "function"
              ? feature.badgeCount()
              : undefined,
        }));

      return {
        id: groupId,
        label: meta.label,
        icon: meta.icon,
        items,
      };
    },
  );

  return orderSidebarSections(sections);
}

export function getSidebarNavigationItems(): SidebarItem[] {
  const items: SidebarItem[] = [
    DASHBOARD_NAV_ITEM,
    ...registeredFeatures.map((feature) => ({
      href: feature.href,
      label: feature.label,
      icon: feature.icon,
      badgeCount:
        typeof feature.badgeCount === "function"
          ? feature.badgeCount()
          : undefined,
    })),
  ];

  return orderSidebarItems(items);
}

export function getDashboardWidgets(): DashboardWidget[] {
  const widgets: DashboardWidget[] = registeredFeatures
    .filter((f): f is FeatureDefinition & { widget: NonNullable<FeatureDefinition["widget"]> } => Boolean(f.widget))
    .map((feature) => ({
      id: feature.id,
      label: feature.label,
      component: feature.widget,
    }));

  return orderDashboardWidgets(widgets);
}

export function getPersistentViews(): PersistentFeatureView[] {
  return registeredFeatures
    .filter(
      (
        f,
      ): f is FeatureDefinition & {
        persistentComponent: NonNullable<FeatureDefinition["persistentComponent"]>;
      } => Boolean(f.persistentComponent),
    )
    .map((feature) => ({
      id: feature.id,
      href: feature.href,
      component: feature.persistentComponent,
    }));
}

export async function initializeBackgroundFeatures(): Promise<void> {
  const tasks = registeredFeatures
    .filter((f) => f.backgroundLoadable && typeof f.init === "function")
    .map((f) => f.init!());

  await Promise.all(tasks);
}

export async function handleSnapshotSync(): Promise<void> {
  const tasks = registeredFeatures
    .filter((f) => typeof f.onSnapshotSync === "function")
    .map((f) => f.onSnapshotSync!());

  await Promise.all(tasks);
}

