import {
  Cpu,
  DesktopPc,
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
import { printerStore } from "../../routes/printers/printers.svelte";
import { serverStore } from "../../routes/servers/servers.svelte";
import { orderDashboardWidgets, orderFeatures, orderSidebarItems } from "./order";
import type { DashboardWidget, FeatureDefinition, SidebarItem, SidebarSection } from "./types";

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
    id: "terminal",
    label: "Terminal",
    href: "/terminal",
    icon: Terminal,
    group: "endpoint",
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
  },
  {
    id: "servers",
    label: "Servers",
    href: "/servers",
    icon: Server,
    badgeCount: () => serverStore.alertCount,
    widget: ServerSummaryCard,
    group: "network",
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

export function getSidebarSections(): SidebarSection[] {
  const groupOrder = ["network", "endpoint"];
  const features = getRegisteredFeatures();

  return groupOrder.map((groupId) => {
    const meta = SIDEBAR_SUBHEADERS[groupId];
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
  });
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
