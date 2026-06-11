import { truncateToWidth } from "@earendil-works/pi-tui";
import type { DailyStats } from "../types.ts";

function formatShortDate(date: string): string {
  const parts = date.split("-");
  return parts.length === 3 ? `${parts[1]}-${parts[2]}` : date;
}

function niceYAxisMax(maxValue: number): number {
  if (maxValue <= 4) return 4;
  if (maxValue <= 10) return 10;
  if (maxValue <= 20) return 20;
  const magnitude = 10 ** Math.floor(Math.log10(maxValue));
  return Math.ceil(maxValue / magnitude) * magnitude;
}

function placeLabel(line: string[], text: string, center: number): void {
  const start = Math.max(0, Math.min(line.length - text.length, center - Math.floor(text.length / 2)));
  for (let i = 0; i < text.length && start + i < line.length; i += 1) {
    line[start + i] = text[i]!;
  }
}

export function ratingTotals(series: DailyStats[]) {
  return series.reduce(
    (acc, day) => ({
      again: acc.again + day.again,
      hard: acc.hard + day.hard,
      good: acc.good + day.good,
      easy: acc.easy + day.easy,
    }),
    { again: 0, hard: 0, good: 0, easy: 0 },
  );
}

export function drawTrendLine(series: DailyStats[], visiblePoints: number, width: number): string[] {
  const plotHeight = 6;
  const rowCount = plotHeight + 1;
  const plotWidth = Math.max(28, width - 8);
  const visibleCount = Math.max(1, Math.min(series.length, visiblePoints));
  const maxReviews = Math.max(0, ...series.map((day) => day.reviews));
  const yMax = niceYAxisMax(maxReviews);
  const midValue = Math.round(yMax / 2);
  const grid = Array.from({ length: rowCount }, () => Array.from({ length: plotWidth }, () => " "));
  const xPositions = series.map((_, index) =>
    series.length === 1 ? 0 : Math.round((index * (plotWidth - 1)) / (series.length - 1)),
  );

  for (let x = 0; x < plotWidth; x += 1) {
    grid[plotHeight]![x] = "─";
  }
  for (const x of xPositions) {
    grid[plotHeight]![x] = "┬";
  }

  const points = series.slice(0, visibleCount).map((day, index) => {
    const x = xPositions[index]!;
    const y = plotHeight - Math.round((day.reviews * plotHeight) / yMax);
    return { x, y };
  });

  for (let i = 0; i < points.length - 1; i += 1) {
    const start = points[i]!;
    const end = points[i + 1]!;
    const midX = Math.floor((start.x + end.x) / 2);

    if (start.y === end.y) {
      for (let x = start.x + 1; x < end.x; x += 1) {
        grid[start.y]![x] = "─";
      }
      continue;
    }

    for (let x = start.x + 1; x < midX; x += 1) {
      grid[start.y]![x] = "─";
    }

    const topY = Math.min(start.y, end.y);
    const bottomY = Math.max(start.y, end.y);
    for (let y = topY + 1; y < bottomY; y += 1) {
      grid[y]![midX] = "│";
    }

    if (end.y < start.y) {
      grid[start.y]![midX] = "╯";
      grid[end.y]![midX] = "╭";
    } else {
      grid[start.y]![midX] = "╮";
      grid[end.y]![midX] = "╰";
    }

    for (let x = midX + 1; x < end.x; x += 1) {
      grid[end.y]![x] = "─";
    }
  }

  for (const point of points) {
    grid[point.y]![point.x] = "●";
  }

  const topLabel = yMax.toString().padStart(3, " ");
  const midLabel = midValue > 0 && midValue < yMax ? midValue.toString().padStart(3, " ") : "   ";
  const lines = grid.map((row, index) => {
    if (index === plotHeight) {
      return `  0 └${row.join("")}`;
    }
    const label = index === 0 ? topLabel : index === Math.floor(plotHeight / 2) ? midLabel : "   ";
    return `${label} │${row.join("")}`;
  });

  const labels = Array.from({ length: plotWidth }, () => " ");
  series.forEach((day, index) => {
    placeLabel(labels, formatShortDate(day.date), xPositions[index]!);
  });
  lines.push(`     ${truncateToWidth(labels.join(""), plotWidth, "", true)}`);
  return lines;
}
