import type { StatusResponse } from "../types.ts";

const ansi = {
  reset: "\x1b[0m",
  dim: "\x1b[2m",
  cyan: "\x1b[36m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
};

function color(value: string, code: string): string {
  return `${code}${value}${ansi.reset}`;
}

function segment(label: string, value: number, colorCode: string): string {
  return `${color(label, colorCode)} ${value}`;
}

export function formatStatusLine(status: StatusResponse): string {
  const prefix = color("Fishword", ansi.dim);
  const deck = color(status.deck.name, ansi.cyan);

  if (status.mode === "empty") {
    return `${prefix} ${deck} · ${color("No cards", ansi.yellow)}`;
  }

  if (status.mode === "complete") {
    return `${prefix} ${deck} · ${segment("Due", status.today.due, ansi.dim)} · ${segment("New", status.today.new_remaining, ansi.dim)} · ${color("Done", ansi.green)} ${status.today.reviewed}`;
  }

  const parts = [
    segment("Due", status.today.due, status.today.due > 0 ? ansi.yellow : ansi.dim),
    segment("New", status.today.new_remaining, status.today.new_remaining > 0 ? ansi.blue : ansi.dim),
  ];
  parts.push(segment("Done", status.today.reviewed, status.today.reviewed > 0 ? ansi.green : ansi.dim));

  return `${prefix} ${deck} · ${parts.join(" · ")}`;
}

export function formatStatusLineMessage(message: "no-deck" | "unavailable"): string {
  const prefix = color("Fishword", ansi.dim);
  const text = message === "no-deck" ? "No deck" : "Unavailable";
  return `${prefix} · ${color(text, ansi.yellow)}`;
}
