import { describe, expect, it } from "vitest";
import {
  formatBytes,
  formatUptime,
  getTemperatureColor,
  getTemperatureStatus,
  getUsageColor,
  getUsageStatus,
} from "./metrics";

describe("endpoint metrics formatting", () => {
  it("formats uptime in human-readable days, hours, and minutes", () => {
    expect(formatUptime(0)).toBe("0m");
    expect(formatUptime(45)).toBe("0m");
    expect(formatUptime(120)).toBe("2m");
    expect(formatUptime(3600)).toBe("1h 0m");
    expect(formatUptime(3660)).toBe("1h 1m");
    expect(formatUptime(86400)).toBe("1d 0h 0m");
    expect(formatUptime(90060)).toBe("1d 1h 1m");
    expect(formatUptime(176400)).toBe("2d 1h 0m");
  });

  it("formats bytes accurately with binary prefixes", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(-100)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(1024 * 1024 * 1.5)).toBe("1.5 MB");
    expect(formatBytes(1024 * 1024 * 1024 * 16)).toBe("16.0 GB");
  });

  it("determines usage status thresholds correctly", () => {
    expect(getUsageStatus(0)).toBe("success");
    expect(getUsageStatus(69.9)).toBe("success");
    expect(getUsageStatus(70)).toBe("warning");
    expect(getUsageStatus(84.9)).toBe("warning");
    expect(getUsageStatus(85)).toBe("error");
    expect(getUsageStatus(100)).toBe("error");
  });

  it("determines usage color thresholds correctly", () => {
    expect(getUsageColor(0)).toBe("var(--color-success)");
    expect(getUsageColor(69.9)).toBe("var(--color-success)");
    expect(getUsageColor(70)).toBe("var(--color-warning)");
    expect(getUsageColor(84.9)).toBe("var(--color-warning)");
    expect(getUsageColor(85)).toBe("var(--color-failure)");
    expect(getUsageColor(100)).toBe("var(--color-failure)");
  });

  it("determines temperature status thresholds correctly", () => {
    expect(getTemperatureStatus(35)).toBe("success");
    expect(getTemperatureStatus(59.9)).toBe("success");
    expect(getTemperatureStatus(60)).toBe("warning");
    expect(getTemperatureStatus(79.9)).toBe("warning");
    expect(getTemperatureStatus(80)).toBe("error");
  });

  it("determines temperature color thresholds correctly", () => {
    expect(getTemperatureColor(35)).toBe("var(--color-success)");
    expect(getTemperatureColor(59.9)).toBe("var(--color-success)");
    expect(getTemperatureColor(60)).toBe("var(--color-warning)");
    expect(getTemperatureColor(79.9)).toBe("var(--color-warning)");
    expect(getTemperatureColor(80)).toBe("var(--color-failure)");
  });
});
