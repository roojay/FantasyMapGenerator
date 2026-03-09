import { useMemo, useRef } from 'react';
import type { MapData, LayerVisibility } from '../types/map';

interface MapViewerProps {
  mapData: MapData | null;
  layers: LayerVisibility;
  colorScheme: 'light' | 'dark';
}

export function MapViewer({ mapData, layers, colorScheme }: MapViewerProps) {
  const svgRef = useRef<SVGSVGElement>(null);

  const bgColor = colorScheme === 'dark' ? '#1a2030' : '#c8e8f8';
  const contourColor = colorScheme === 'dark' ? '#4a5568' : '#8899aa';
  const riverColor = colorScheme === 'dark' ? '#63b3ed' : '#4488cc';
  const slopeColor = colorScheme === 'dark' ? '#a0aec0' : '#555566';
  const territoryColor = colorScheme === 'dark' ? '#fc8181' : '#cc4444';
  const cityColor = colorScheme === 'dark' ? '#e2e8f0' : '#222233';
  const townColor = colorScheme === 'dark' ? '#90cdf4' : '#445566';
  const labelColor = colorScheme === 'dark' ? '#f7fafc' : '#111122';

  const svgContent = useMemo(() => {
    if (!mapData) return null;
    const w = 1920;
    const h = 1080;

    const polylineToPath = (coords: number[]) => {
      if (coords.length < 4) return '';
      const parts: string[] = [];
      for (let i = 0; i < coords.length; i += 2) {
        const x = (coords[i] * w).toFixed(1);
        const y = (coords[i + 1] * h).toFixed(1);
        parts.push(i === 0 ? `M${x},${y}` : `L${x},${y}`);
      }
      return parts.join(' ');
    };

    const mergedPath = (polylines: number[][]) =>
      polylines.map(polylineToPath).filter(Boolean).join(' ');

    return { w, h, mergedPath };
  }, [mapData]);

  if (!mapData || !svgContent) {
    return (
      <div className="flex-1 flex items-center justify-center" style={{ backgroundColor: bgColor }}>
        <div className="text-center text-gray-400">
          <div className="text-6xl mb-4">🗺️</div>
          <div className="text-lg">No map data loaded</div>
        </div>
      </div>
    );
  }

  const { w, h, mergedPath } = svgContent;

  return (
    <div className="flex-1 overflow-hidden">
      <svg
        ref={svgRef}
        viewBox={`0 0 ${w} ${h}`}
        className="w-full h-full"
        style={{ backgroundColor: bgColor }}
        preserveAspectRatio="xMidYMid meet"
      >
        {layers.contour && (
          <g id="contour">
            <path
              d={mergedPath(mapData.contour)}
              stroke={contourColor}
              strokeWidth="0.5"
              fill="none"
            />
          </g>
        )}

        {layers.rivers && (
          <g id="rivers">
            <path
              d={mergedPath(mapData.river)}
              stroke={riverColor}
              strokeWidth="1.0"
              fill="none"
            />
          </g>
        )}

        {layers.slopes && (
          <g id="slopes" stroke={slopeColor} strokeWidth="0.7">
            {Array.from({ length: Math.floor(mapData.slope.length / 4) }, (_, i) => (
              <line
                key={i}
                x1={(mapData.slope[i * 4] * w).toFixed(1)}
                y1={(mapData.slope[i * 4 + 1] * h).toFixed(1)}
                x2={(mapData.slope[i * 4 + 2] * w).toFixed(1)}
                y2={(mapData.slope[i * 4 + 3] * h).toFixed(1)}
              />
            ))}
          </g>
        )}

        {layers.territory && (
          <g id="territory">
            <path
              d={mergedPath(mapData.territory)}
              stroke={territoryColor}
              strokeWidth="1.0"
              fill="none"
              strokeDasharray="4,3"
            />
          </g>
        )}

        {layers.cities && (
          <g id="cities" fill={cityColor}>
            {Array.from({ length: Math.floor(mapData.city.length / 2) }, (_, i) => (
              <circle
                key={i}
                cx={(mapData.city[i * 2] * w).toFixed(1)}
                cy={(mapData.city[i * 2 + 1] * h).toFixed(1)}
                r="5"
              />
            ))}
          </g>
        )}

        {layers.towns && (
          <g id="towns" fill={townColor}>
            {Array.from({ length: Math.floor(mapData.town.length / 2) }, (_, i) => (
              <circle
                key={i}
                cx={(mapData.town[i * 2] * w).toFixed(1)}
                cy={(mapData.town[i * 2 + 1] * h).toFixed(1)}
                r="3"
              />
            ))}
          </g>
        )}

        {layers.labels && (
          <g id="labels" fontFamily="serif">
            {mapData.label.map((lbl, i) => (
              <text
                key={i}
                x={(lbl.position[0] * w).toFixed(1)}
                y={(lbl.position[1] * h).toFixed(1)}
                fontSize={Math.max(8, lbl.fontsize * mapData.draw_scale)}
                fill={labelColor}
                textAnchor="middle"
              >
                {lbl.text}
              </text>
            ))}
          </g>
        )}
      </svg>
    </div>
  );
}
