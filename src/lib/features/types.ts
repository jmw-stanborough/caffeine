import type { Component } from "svelte";

export interface FeatureDefinition {
  id: string;
  label: string;
  href: string;
  icon: Component<{ size?: number | string; class?: string }>;
  badgeCount?: () => number | undefined;
  widget?: Component;
  group?: string;
}

export interface SidebarItem {
  href: string;
  label: string;
  icon: Component<{ size?: number | string; class?: string }>;
  badgeCount?: number;
}

export interface SidebarSection {
  id: string;
  label: string;
  icon: Component<{ size?: number | string; class?: string }>;
  items: SidebarItem[];
}

export interface DashboardWidget {
  id: string;
  label: string;
  component: Component;
}
