import type { Component } from "svelte";

export interface FeatureDefinition {
  id: string;
  label: string;
  href: string;
  icon: Component<{ size?: number | string; class?: string }>;
  badgeCount?: () => number | undefined;
  widget?: Component;
  widgetLabel?: string;
  group?: string;
  backgroundLoadable?: boolean;
  init?: () => Promise<void> | void;
  onSnapshotSync?: () => Promise<void> | void;
  persistentComponent?: Component<{ active: boolean }>;
}

export interface PersistentFeatureView {
  id: string;
  href: string;
  component: Component<{ active: boolean }>;
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
  badgeCount?: number;
}

export interface DashboardWidget {
  id: string;
  label: string;
  component: Component;
}
