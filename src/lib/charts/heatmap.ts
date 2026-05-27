import * as d3 from "d3";

export interface HeatmapOptions {
  width?: number;
  height?: number;
  margin?: { top: number; right: number; bottom: number; left: number };
  expectedRuns?: Map<string, number>;
}

const STATE_LABELS: Record<string, string> = {
  "000": "Empty",
  "100": "1B",
  "010": "2B",
  "001": "3B",
  "110": "1B 2B",
  "101": "1B 3B",
  "011": "2B 3B",
  "111": "Loaded",
};

function friendlyState(raw: string): string {
  const match = raw.match(/^(\d)_(.+)$/);
  if (!match) return raw;
  const outs = match[1];
  const bases = match[2];
  if (bases === "---") return "3 Outs";
  const baseName = STATE_LABELS[bases] ?? bases;
  return `${outs} out, ${baseName}`;
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
    expectedRuns,
  } = opts;

  const innerW = width - margin.left - margin.right;
  const innerH = height - margin.top - margin.bottom;
  const n = states.length;

  d3.select(container).selectAll("*").remove();

  const tooltip = d3
    .select(container)
    .append("div")
    .style("position", "absolute")
    .style("pointer-events", "none")
    .style("background", "var(--bg-elev)")
    .style("border", "1px solid var(--line)")
    .style("border-radius", "6px")
    .style("padding", "8px 12px")
    .style("font-size", "13px")
    .style("line-height", "1.4")
    .style("box-shadow", "var(--shadow)")
    .style("color", "var(--ink)")
    .style("opacity", "0")
    .style("z-index", "30")
    .style("white-space", "nowrap");

  d3.select(container).style("position", "relative");

  const svg = d3
    .select(container)
    .append("svg")
    .attr("width", "100%")
    .attr("viewBox", `0 0 ${width} ${height}`)
    .attr("preserveAspectRatio", "xMinYMin meet")
    .style("display", "block");

  const g = svg
    .append("g")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  const cellW = innerW / n;
  const cellH = innerH / n;

  const maxVal = d3.max(matrix.flat().filter((v) => v > 0)) || 1;
  const color = d3
    .scaleSequential(d3.interpolateYlOrRd)
    .domain([0, maxVal]);

  const isLarge = n > 10;

  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      const val = matrix[i][j];
      if (val === 0) continue;

      const fromLabel = isLarge
        ? friendlyState(labels?.[i] ?? states[i])
        : (labels?.[i] ?? states[i]);
      const toLabel = isLarge
        ? friendlyState(labels?.[j] ?? states[j])
        : (labels?.[j] ?? states[j]);

      g.append("rect")
        .attr("x", j * cellW)
        .attr("y", i * cellH)
        .attr("width", cellW - 0.5)
        .attr("height", cellH - 0.5)
        .attr("fill", color(val))
        .attr("rx", 1)
        .style("cursor", "crosshair")
        .on("mouseenter", function (event) {
          d3.select(this)
            .attr("stroke", "var(--ink)")
            .attr("stroke-width", 1.5);
          let html =
            `<strong>${fromLabel}</strong> → <strong>${toLabel}</strong><br/>` +
            `Probability: <strong>${(val * 100).toFixed(1)}%</strong>`;
          if (expectedRuns) {
            const fromER = expectedRuns.get(states[i]);
            const toER = expectedRuns.get(states[j]);
            if (fromER !== undefined && toER !== undefined) {
              html += `<br/>Expected runs: <strong>${fromER.toFixed(3)}</strong> → <strong>${toER.toFixed(3)}</strong>`;
            }
          }
          tooltip.html(html).style("opacity", "1");
        })
        .on("mousemove", function (event) {
          const rect = container.getBoundingClientRect();
          const tipNode = tooltip.node() as HTMLElement;
          const tipW = tipNode?.offsetWidth ?? 200;
          const mouseX = event.clientX - rect.left;
          const mouseY = event.clientY - rect.top;
          const flipX = mouseX + tipW + 20 > rect.width;
          const x = flipX ? mouseX - tipW - 12 : mouseX + 12;
          const y = mouseY - 10;
          tooltip.style("left", x + "px").style("top", y + "px");
        })
        .on("mouseleave", function () {
          d3.select(this).attr("stroke", "none");
          tooltip.style("opacity", "0");
        });
    }
  }

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

  if (isLarge) {
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
}
