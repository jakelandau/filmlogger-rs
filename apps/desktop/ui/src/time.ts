// Conversions between Rust's RFC 3339 UTC timestamps and wall-clock values in
// a selected IANA time zone. `datetime-local` inputs carry no zone, so the
// caller chooses one and DST is resolved by `Intl`.

export function systemTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}

function supportedTimeZones(): string[] | null {
  const intl: unknown = Intl;
  if (typeof intl !== "object" || intl === null || !("supportedValuesOf" in intl)) {
    return null;
  }
  const supported = intl.supportedValuesOf;
  if (typeof supported !== "function") return null;
  const values: unknown = supported.call(Intl, "timeZone");
  if (!Array.isArray(values)) return null;
  return values.filter((value): value is string => typeof value === "string");
}

const CONTINENT_AREAS: Record<string, true> = {
  Africa: true,
  America: true,
  Antarctica: true,
  Arctic: true,
  Asia: true,
  Atlantic: true,
  Australia: true,
  Europe: true,
  Indian: true,
  Pacific: true,
};

function isContinentCity(zone: string): boolean {
  const slash = zone.indexOf("/");
  return (
    slash > 0 &&
    slash < zone.length - 1 &&
    CONTINENT_AREAS[zone.slice(0, slash)] === true
  );
}

let timeZoneCache: string[] | null = null;

export function availableTimeZones(): string[] {
  if (timeZoneCache != null) return timeZoneCache;
  const zones = (supportedTimeZones() ?? [systemTimeZone()]).filter(isContinentCity);
  timeZoneCache = [...new Set(zones)].sort();
  return timeZoneCache;
}

/** Options for the searchable field, keeping an older zone selectable. */
export function timeZoneOptions(current: string): string[] {
  const zones = availableTimeZones();
  return current !== "" && !zones.includes(current) ? [current, ...zones] : zones;
}

function formatter(timeZone: string | null | undefined): Intl.DateTimeFormat {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone: timeZone ?? undefined,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
  });
}

function wallParts(iso: string, timeZone: string | null | undefined): Record<string, string> | null {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return null;
  try {
    return Object.fromEntries(
      formatter(timeZone)
        .formatToParts(d)
        .filter((part) => part.type !== "literal")
        .map((part) => [part.type, part.value])
    );
  } catch {
    return null;
  }
}

// `iso` is an RFC 3339 UTC string; returns `YYYY-MM-DDTHH:MM` in `timeZone`.
export function toLocalInput(iso: string, timeZone?: string | null): string {
  const parts = wallParts(iso, timeZone);
  if (parts == null) return "";
  return `${parts.year}-${parts.month}-${parts.day}T${parts.hour}:${parts.minute}`;
}

function zoneOffsetMs(date: Date, timeZone: string): number {
  const parts = Object.fromEntries(
    new Intl.DateTimeFormat("en-US", {
      timeZone,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hourCycle: "h23",
    })
      .formatToParts(date)
      .filter((part) => part.type !== "literal")
      .map((part) => [part.type, part.value])
  );
  const asUtc = Date.UTC(
    Number(parts.year),
    Number(parts.month) - 1,
    Number(parts.day),
    Number(parts.hour),
    Number(parts.minute),
    Number(parts.second)
  );
  return asUtc - Math.floor(date.getTime() / 1000) * 1000;
}

// `value` is a `datetime-local` value; returns RFC 3339 UTC for `timeZone`.
// Returns "" for an invalid wall time (including a DST spring-forward gap).
export function fromLocalInput(value: string, timeZone?: string | null): string {
  if (timeZone == null || timeZone === "") {
    return new Date(value).toISOString();
  }
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?$/.exec(value);
  if (match == null) return "";
  const [, year, month, day, hour, minute, second = "00"] = match;
  try {
    const wallUtc = Date.UTC(
      Number(year),
      Number(month) - 1,
      Number(day),
      Number(hour),
      Number(minute),
      Number(second)
    );
    let instant = new Date(wallUtc - zoneOffsetMs(new Date(wallUtc), timeZone));
    const corrected = zoneOffsetMs(instant, timeZone);
    if (corrected !== zoneOffsetMs(new Date(wallUtc), timeZone)) {
      instant = new Date(wallUtc - corrected);
    }
    // A nonexistent local time normalizes to a different wall clock.
    if (toLocalInput(instant.toISOString(), timeZone) !== value.slice(0, 16)) {
      return "";
    }
    return instant.toISOString();
  } catch {
    return "";
  }
}

export function formatDisplay(iso: string, timeZone?: string | null): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  try {
    return d.toLocaleString(undefined, {
      timeZone: timeZone ?? undefined,
      dateStyle: "medium",
      timeStyle: "short",
      timeZoneName: "short",
    });
  } catch {
    return d.toLocaleString();
  }
}

export function formatYear(iso: string): string | null {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? null : String(d.getUTCFullYear());
}
