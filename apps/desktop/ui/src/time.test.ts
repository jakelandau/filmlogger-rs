import { describe, expect, it } from "vitest";
import { availableTimeZones, fromLocalInput, toLocalInput } from "./time";

describe("time zone conversion", () => {
  it("renders an instant in a selected zone", () => {
    expect(toLocalInput("2020-06-01T12:00:00Z", "America/New_York")).toBe(
      "2020-06-01T08:00"
    );
    expect(toLocalInput("2020-01-01T12:00:00Z", "America/New_York")).toBe(
      "2020-01-01T07:00"
    );
  });

  it("converts wall time through the zone's standard and daylight offsets", () => {
    expect(fromLocalInput("2020-06-01T08:00", "America/New_York")).toBe(
      "2020-06-01T12:00:00.000Z"
    );
    expect(fromLocalInput("2020-01-01T07:00", "America/New_York")).toBe(
      "2020-01-01T12:00:00.000Z"
    );
  });

  it("rejects a spring-forward gap", () => {
    expect(fromLocalInput("2021-03-14T02:30", "America/New_York")).toBe("");
  });

  it("accepts the first occurrence of a fall-back overlap", () => {
    expect(fromLocalInput("2021-11-07T01:30", "America/New_York")).toBe(
      "2021-11-07T05:30:00.000Z"
    );
  });

  it("offers only continent/city IANA zones", () => {
    const zones = availableTimeZones();
    expect(zones).toContain("Europe/London");
    expect(zones).toContain("America/New_York");
    expect(zones).not.toContain("ACT");
    expect(zones).not.toContain("EST");
    expect(zones).not.toContain("UTC");
    expect(zones.every((zone) => zone.includes("/"))).toBe(true);
  });
});
