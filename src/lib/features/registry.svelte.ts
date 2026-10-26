import {
  DesktopPc,
  LayoutDashboard,
  Printer,
  Server,
} from "$lib/components/icons";
import EndpointSummaryCard from "$lib/components/endpoint-summary-card.svelte";
import PrinterSummaryCard from "$lib/components/printer-summary-card.svelte";
import ServerSummaryCard from "$lib/components/server-summary-card.svelte";
import { printerStore } from "../../routes/printers/printers.svelte";
import { serverStore } from "../../routes/servers/servers.svelte";
import { orderDashboardWidgets, orderFeatures, orderSidebarItems } from "./order";
import type { DashboardWidget, FeatureDefinition, SidebarItem } from "./types";

export const DASHBOARD_NAV_ITEM: SidebarItem = {
  href: "/",
  label: "Dashboard",
  icon: LayoutDashboard,
};

const registeredFeatures = $state<FeatureDefinition[]>([
  {
    id: "endpoint",
    label: "Endpoint",
    href: "/endpoint",
    icon: DesktopPc,
    widget: EndpointSummaryCard,
  },
  {
    id: "printers",
    label: "Printers",
    href: "/printers",
    icon: Printer,
    badgeCount: () => printerStore.alertCount,
    widget: PrinterSummaryCard,
  },
  {
    id: "servers",
    label: "Servers",
    href: "/servers",
    icon: Server,
    badgeCount: () => serverStore.alertCount,
    widget: ServerSummaryCard,
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
