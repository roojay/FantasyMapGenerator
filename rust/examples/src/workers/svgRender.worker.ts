import { buildStandardMapSvg } from "@/lib/standardMapSvg";
import type { MapLayers } from "@/types/map";

interface BuildSvgRequest {
  type: "build-svg";
  requestId: number;
  mapJson: string;
  layers: MapLayers;
}

interface BuildSvgSuccessResponse {
  type: "success";
  requestId: number;
  svgMarkup: string;
}

interface BuildSvgErrorResponse {
  type: "error";
  requestId: number;
  error: string;
}

type BuildSvgResponse = BuildSvgSuccessResponse | BuildSvgErrorResponse;

const workerScope = self as typeof self & {
  postMessage: (message: BuildSvgResponse) => void;
};

self.onmessage = async (event: MessageEvent<BuildSvgRequest>) => {
  const { type, requestId, mapJson, layers } = event.data;
  if (type !== "build-svg") {
    return;
  }

  try {
    const svgMarkup = await buildStandardMapSvg(mapJson, layers);

    workerScope.postMessage({
      type: "success",
      requestId,
      svgMarkup,
    });
  } catch (error) {
    workerScope.postMessage({
      type: "error",
      requestId,
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
