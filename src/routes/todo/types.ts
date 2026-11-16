export type RecurrenceRule = "none" | "daily" | "weekly" | "monthly";
export type TodoStatus = "success" | "warning" | "error" | "neutral";

export interface TodoItem {
  id: string;
  title: string;
  completed: boolean;
  completed_at: number | null;
  due_at: number | null;
  reminder_at: number | null;
  reminder_sent: boolean;
  reminder_acknowledged: boolean;
  recurrence: RecurrenceRule | null;
  created_at: number;
  sort_order: number;
}

export interface CreateTodoInput {
  title: string;
  due_at: number | null;
  reminder_at: number | null;
  recurrence: RecurrenceRule | null;
}

export interface UpdateTodoInput extends CreateTodoInput {
  id: string;
}

export interface TodoCategories {
  overdue: TodoItem[];
  active: TodoItem[];
  completed: TodoItem[];
}

export interface TodoStatusCounts {
  online: number;
  degraded: number;
  offline: number;
  checking: number;
}

export interface TodoAttentionItem {
  id: string;
  label: string;
  detail: string;
  severity: "offline" | "degraded";
}
