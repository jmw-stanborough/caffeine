import type { ITheme } from "@xterm/xterm";

const GRUVBOX_DARK: ITheme = {
  background: "#1d2021",
  foreground: "#ebdbb2",
  cursor: "#fabd2f",
  cursorAccent: "#1d2021",
  selectionBackground: "#504945",
  selectionForeground: "#ebdbb2",
  black: "#282828",
  red: "#cc241d",
  green: "#98971a",
  yellow: "#d79921",
  blue: "#458588",
  magenta: "#b16286",
  cyan: "#689d6a",
  white: "#a89984",
  brightBlack: "#928374",
  brightRed: "#fb4934",
  brightGreen: "#b8bb26",
  brightYellow: "#fabd2f",
  brightBlue: "#83a598",
  brightMagenta: "#d3869b",
  brightCyan: "#8ec07c",
  brightWhite: "#ebdbb2",
};

const GRUVBOX_LIGHT: ITheme = {
  background: "#fbf1c7",
  foreground: "#3c3836",
  cursor: "#3c3836",
  cursorAccent: "#fbf1c7",
  selectionBackground: "#d5c4a1",
  selectionForeground: "#3c3836",
  black: "#3c3836",
  red: "#cc241d",
  green: "#79740e",
  yellow: "#b57614",
  blue: "#076678",
  magenta: "#8f3f71",
  cyan: "#427b58",
  white: "#ebdbb2",
  brightBlack: "#928374",
  brightRed: "#9d0006",
  brightGreen: "#b8bb26",
  brightYellow: "#b57614",
  brightBlue: "#458588",
  brightMagenta: "#d3869b",
  brightCyan: "#689d6a",
  brightWhite: "#fbf1c7",
};

export function getTerminalTheme(isDark: boolean): ITheme {
  return isDark ? GRUVBOX_DARK : GRUVBOX_LIGHT;
}
