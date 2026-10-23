export type SortDirection = "asc" | "desc";

export interface SortDirectionLabels {
  asc: string;
  desc: string;
}

export interface SortFieldDefinition<T, TContext = unknown> {
  id: string;
  label: string;
  getValue?: (item: T, context?: TContext) => unknown;
  compare?: (a: T, b: T, direction: SortDirection, context?: TContext) => number;
  directionLabels?: SortDirectionLabels;
  defaultDirection?: SortDirection;
}

export interface SortRule {
  id: string;
  fieldId: string;
  direction: SortDirection;
}
