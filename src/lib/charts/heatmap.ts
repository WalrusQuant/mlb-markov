import * as d3 from "d3";

export interface HeatmapOptions {
  width?: number;
  height?: number;
  margin?: { top: number; right: number; bottom: number; left: number };
  colorScheme?: "sequential" | "diverging";
}

export function renderHeatmap(
  container: HTMLElement,
  states: string[],
  matrix: number[][],
  labels?: string[],
  opts: HeatmapOptions = {},
): void {
  const {
    width = 700,
    height = 700,
    margin = { top: 80, right: 20, bottom: 20, left: 80 },
  } = opts;

  const innerW = width - margin.left - margin.right;
  const innerH = height - margin.top - margin.bottom;
  const n = states.length;

  // Clear previous
  d3.select(container).selectAll("*").remove();

  const svg = d3
    .select(container)
    .append("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", `0 0 ${width} ${height}`);

  const g = svg
    .append("g")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  const cellW = innerW / n;
  const cellH = innerH / n;

  // Color scale
  const maxVal = d3.max(matrix.flat().filter((v) => v > 0)) || 1;
  const color = d3
    .scaleSequential(d3.interpolateYlOrRd)
    .domain([0, maxVal]);

  // Draw cells
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      const val = matrix[i][j];
      if (val === 0) continue;

      g.append("rect")
        .attr("x", j * cellW)
        .attr("y", i * cellH)
        .attr("width", cellW - 0.5)
        .attr("height", cellH - 0.5)
        .attr("fill", color(val))
        .attr("rx", 1)
        .append("title")
        .text(
          `${labels?.[i] ?? states[i]} → ${labels?.[j] ?? states[j]}\nP = ${val.toFixed(4)}`,
        );
    }
  }

  // X-axis labels (top)
  const shortLabels = states.map((s) => s.replace(/^[012]_/, ""));
  const outGroups = ["0 out", "1 out", "2 out"];

  g.selectAll(".x-label")
    .data(states)
    .enter()
    .append("text")
    .attr("class", "x-label")
    .attr("x", (_, i) => i * cellW + cellW / 2)
    .attr("y", -8)
    .attr("text-anchor", "middle")
    .attr("font-size", n > 20 ? "7px" : "9px")
    .attr("fill", "var(--ink-mute)")
    .text((_, i) => shortLabels[i]);

  // Y-axis labels (left)
  g.selectAll(".y-label")
    .data(states)
    .enter()
    .append("text")
    .attr("class", "y-label")
    .attr("x", -8)
    .attr("y", (_, i) => i * cellH + cellH / 2)
    .attr("text-anchor", "end")
    .attr("dominant-baseline", "middle")
    .attr("font-size", n > 20 ? "7px" : "9px")
    .attr("fill", "var(--ink-mute)")
    .text((_, i) => shortLabels[i]);

  // Out group brackets on y-axis
  for (let o = 0; o < 3; o++) {
    const y = o * 8 * cellH;
    g.append("text")
      .attr("x", -margin.left + 4)
      .attr("y", y + 4 * cellH)
      .attr("text-anchor", "start")
      .attr("dominant-baseline", "middle")
      .attr("font-size", "9px")
      .attr("font-weight", "600")
      .attr("fill", "var(--ink-soft)")
      .text(outGroups[o]);
  }

  // Grid lines between out groups
  for (let o = 1; o <= 3; o++) {
    const pos = o * 8 * cellH;
    g.append("line")
      .attr("x1", 0)
      .attr("x2", innerW)
      .attr("y1", pos)
      .attr("y2", pos)
      .attr("stroke", "var(--line)")
      .attr("stroke-width", 1);
    g.append("line")
      .attr("x1", pos * (innerW / innerH))
      .attr("x2", pos * (innerW / innerH))
      .attr("y1", 0)
      .attr("y2", innerH)
      .attr("stroke", "var(--line)")
      .attr("stroke-width", 1);
  }
}
