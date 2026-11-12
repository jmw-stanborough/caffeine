import { describe, expect, it } from "vitest";
import { resolvePrinterUrl } from "./url";
import type { Printer } from "./types";

describe("resolvePrinterUrl", () => {
  it("defaults to http scheme using printer host when web_url is omitted", () => {
    const printer: Printer = {
      id: "p1",
      host: "192.168.1.50",
      label: "Office Printer",
      snmp_community: "public",
    };
    expect(resolvePrinterUrl(printer)).toBe("http://192.168.1.50");
  });

  it("preserves explicit https scheme in web_url", () => {
    const printer: Printer = {
      id: "p2",
      host: "10.0.0.12",
      label: "Color Laser",
      snmp_community: "public",
      web_url: "https://10.0.0.12:8443",
    };
    expect(resolvePrinterUrl(printer)).toBe("https://10.0.0.12:8443");
  });

  it("preserves explicit http scheme in web_url", () => {
    const printer: Printer = {
      id: "p3",
      host: "10.0.0.13",
      label: "Mono Laser",
      snmp_community: "public",
      web_url: "http://printer.local/admin",
    };
    expect(resolvePrinterUrl(printer)).toBe("http://printer.local/admin");
  });

  it("trims whitespace from raw url strings", () => {
    const printer: Printer = {
      id: "p4",
      host: "  192.168.1.99  ",
      label: "Deskjet",
      snmp_community: "public",
    };
    expect(resolvePrinterUrl(printer)).toBe("http://192.168.1.99");
  });
});
