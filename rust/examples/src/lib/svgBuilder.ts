import type { MapData, MapLayers } from "@/types/map";

interface RenderStyles {
  BACKGROUND_RGBA: string;
  SLOPE_RGBA: string;
  RIVER_RGBA: string;
  CONTOUR_RGBA: string;
  BORDER_RGBA: string;
  CITY_MARKER_RGBA: string;
  TOWN_MARKER_RGBA: string;
  TEXT_RGBA: string;
  SLOPE_LINE_WIDTH: number;
  RIVER_LINE_WIDTH: number;
  CONTOUR_LINE_WIDTH: number;
  BORDER_LINE_WIDTH: number;
  BORDER_DASH_PATTERN: string;
  CITY_MARKER_OUTER_RADIUS: number;
  CITY_MARKER_INNER_RADIUS: number;
  TOWN_MARKER_RADIUS: number;
}

const DEFAULT_LAYERS: MapLayers = {
  slope: true,
  river: true,
  contour: true,
  border: true,
  city: true,
  town: true,
  label: true
};

const round = (value: number) => Math.round(value * 10) / 10;

export function generateSVGString(
  mapData: MapData,
  styles: RenderStyles,
  layers: MapLayers = DEFAULT_LAYERS
) {
  const enabledLayers = { ...DEFAULT_LAYERS, ...layers };
  const width = mapData.image_width;
  const height = mapData.image_height;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" preserveAspectRatio="xMidYMid meet" style="display:block">`;
  svg += `<rect width="${width}" height="${height}" fill="${styles.BACKGROUND_RGBA}"/>`;

  if (enabledLayers.slope && mapData.slope.length > 0) {
    let d = "";
    for (let index = 0; index < mapData.slope.length; index += 4) {
      const x1 = round(mapData.slope[index] * width);
      const y1 = round(height - mapData.slope[index + 1] * height);
      const x2 = round(mapData.slope[index + 2] * width);
      const y2 = round(height - mapData.slope[index + 3] * height);
      d += `M${x1} ${y1}L${x2} ${y2}`;
    }
    svg += `<path stroke="${styles.SLOPE_RGBA}" stroke-width="${round(styles.SLOPE_LINE_WIDTH)}" d="${d}"/>`;
  }

  if (enabledLayers.border && mapData.territory.length > 0) {
    const paths = createSVGPaths(mapData.territory, width, height);
    svg += `<g fill="none" stroke="${styles.BACKGROUND_RGBA}" stroke-width="${round(styles.BORDER_LINE_WIDTH)}">${paths}</g>`;
    svg += `<g fill="none" stroke="${styles.BORDER_RGBA}" stroke-width="${round(styles.BORDER_LINE_WIDTH)}" stroke-dasharray="${styles.BORDER_DASH_PATTERN}" stroke-linecap="butt" stroke-linejoin="bevel">${paths}</g>`;
  }

  if (enabledLayers.river && mapData.river.length > 0) {
    svg += `<g fill="none" stroke="${styles.RIVER_RGBA}" stroke-width="${round(styles.RIVER_LINE_WIDTH)}">${createSVGPaths(mapData.river, width, height)}</g>`;
  }

  if (enabledLayers.contour && mapData.contour.length > 0) {
    svg += `<g fill="none" stroke="${styles.CONTOUR_RGBA}" stroke-width="${round(styles.CONTOUR_LINE_WIDTH)}" stroke-linecap="round" stroke-linejoin="round">${createSVGPaths(mapData.contour, width, height)}</g>`;
  }

  if (enabledLayers.city && mapData.city.length > 0) {
    svg += "<g>";
    for (let index = 0; index < mapData.city.length; index += 2) {
      const x = round(mapData.city[index] * width);
      const y = round(height - mapData.city[index + 1] * height);
      svg += `<circle cx="${x}" cy="${y}" r="${round(styles.CITY_MARKER_OUTER_RADIUS)}" fill="${styles.BACKGROUND_RGBA}" stroke="${styles.CITY_MARKER_RGBA}" stroke-width="${round(styles.SLOPE_LINE_WIDTH)}"/>`;
      svg += `<circle cx="${x}" cy="${y}" r="${round(styles.CITY_MARKER_INNER_RADIUS)}" fill="${styles.CITY_MARKER_RGBA}"/>`;
    }
    svg += "</g>";
  }

  if (enabledLayers.town && mapData.town.length > 0) {
    svg += `<g fill="${styles.BACKGROUND_RGBA}" stroke="${styles.TOWN_MARKER_RGBA}" stroke-width="${round(styles.SLOPE_LINE_WIDTH)}">`;
    for (let index = 0; index < mapData.town.length; index += 2) {
      const x = round(mapData.town[index] * width);
      const y = round(height - mapData.town[index + 1] * height);
      svg += `<circle cx="${x}" cy="${y}" r="${round(styles.TOWN_MARKER_RADIUS)}"/>`;
    }
    svg += "</g>";
  }

  if (enabledLayers.label && mapData.label.length > 0) {
    svg += `<g>`;
    const fontMap: Record<string, string> = { "Times New Roman": "serif" };
    for (const label of mapData.label) {
      const x = round(label.position[0] * width);
      const y = round(height - label.position[1] * height);
      const fontFamily = fontMap[label.fontface] ?? label.fontface ?? "serif";
      const safeText = label.text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
      svg += `<text x="${x}" y="${y}" font-family="${fontFamily}" font-size="${label.fontsize}" fill="white" stroke="white" stroke-width="3" stroke-linejoin="round" paint-order="stroke fill" text-anchor="start" dominant-baseline="alphabetic">${safeText}</text>`;
      svg += `<text x="${x}" y="${y}" font-family="${fontFamily}" font-size="${label.fontsize}" fill="${styles.TEXT_RGBA}" text-anchor="start" dominant-baseline="alphabetic">${safeText}</text>`;
    }
    svg += "</g>";
  }

  svg += "</svg>";
  return svg;
}

function createSVGPaths(data: number[][], width: number, height: number) {
  let output = "";
  for (const path of data) {
    if (path.length < 2) {
      continue;
    }

    let d = `M${round(path[0] * width)} ${round(height - path[1] * height)}`;
    for (let index = 2; index < path.length; index += 2) {
      d += ` ${round(path[index] * width)} ${round(height - path[index + 1] * height)}`;
    }
    output += `<path d="${d}"/>`;
  }

  return output;
}
