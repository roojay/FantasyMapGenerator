import { generateSVGString } from "@/lib/svgBuilder";
import { generateSatelliteSVG } from "@/lib/svgTerrainBuilder";
import type { MapData, MapLayers, RenderBackend, RendererPreference } from "@/types/map";

const DEFAULT_LAYERS: MapLayers = {
  slope: true,
  river: true,
  contour: true,
  border: true,
  city: true,
  town: true,
  label: true
};

interface EncodedRasterTexture {
  width: number;
  height: number;
  texelSize: [number, number];
  data: Uint8Array;
}

interface SatelliteTextureBundle {
  mapData: MapData;
  height: EncodedRasterTexture;
  land: EncodedRasterTexture;
  flux: EncodedRasterTexture;
}

export class FantasyMapCanvasRenderer {
  private stage: HTMLDivElement;
  private canvas: HTMLCanvasElement;
  private device: GPUDevice | null = null;
  private context:
    | GPUCanvasContext
    | WebGLRenderingContext
    | WebGL2RenderingContext
    | CanvasRenderingContext2D
    | null = null;
  private svgPreviewImage: HTMLImageElement | null = null;
  private svgPreviewUrl: string | null = null;
  private svgPreviewVersion = 0;
  private currentSvgWidth = 0;
  private currentSvgHeight = 0;
  private renderMode: RenderBackend | null = null;
  private availableModes: RenderBackend[] = [];
  private mapData: MapData | null = null;
  private layers: MapLayers = { ...DEFAULT_LAYERS };
  private useSatelliteStyle = false; // 新增：是否使用卫星风格
  private satelliteTextureCache: SatelliteTextureBundle | null = null;

  private BACKGROUND_RGBA = [1, 1, 1, 1] as const;
  private SLOPE_RGBA = [0, 0, 0, 0.75] as const;
  private RIVER_RGBA = [0, 0, 0, 1] as const;
  private CONTOUR_RGBA = [0, 0, 0, 1] as const;
  private BORDER_RGBA = [0, 0, 0, 1] as const;
  private CITY_MARKER_RGBA = [0, 0, 0, 1] as const;
  private TOWN_MARKER_RGBA = [0, 0, 0, 1] as const;
  private TEXT_RGBA = [0, 0, 0, 1] as const;

  private SLOPE_LINE_WIDTH = 1;
  private RIVER_LINE_WIDTH = 2.5;
  private CONTOUR_LINE_WIDTH = 1.5;
  private BORDER_LINE_WIDTH = 6;
  private BORDER_DASH_PATTERN = [3, 4];
  private CITY_MARKER_OUTER_RADIUS = 10;
  private CITY_MARKER_INNER_RADIUS = 5;
  private TOWN_MARKER_RADIUS = 5;

  constructor(stage: HTMLDivElement) {
    this.stage = stage;
    this.canvas = this.createCanvas();
  }

  get currentMode() {
    return this.renderMode;
  }

  getMapSize() {
    if (!this.mapData) {
      return null;
    }

    return {
      width: this.mapData.image_width,
      height: this.mapData.image_height
    };
  }

  async initialize(preferredMode: RendererPreference = "auto") {
    await this.detectAvailableModes();

    const candidates =
      preferredMode === "auto"
        ? this.availableModes
        : [preferredMode].filter((mode): mode is RenderBackend =>
            this.availableModes.includes(mode as RenderBackend)
          );

    for (const mode of candidates) {
      this.replaceCanvas();
      if (await this.initializeMode(mode)) {
        return {
          mode,
          availableModes: [...this.availableModes]
        };
      }
    }

    throw new Error("No rendering backend available");
  }

  destroy() {
    this.cleanup();
    this.canvas.remove();
  }

  cleanup() {
    if (this.device) {
      this.device.destroy();
      this.device = null;
    }

    if (this.context && this.renderMode === "webgl") {
      const loseContext = (this.context as WebGLRenderingContext | WebGL2RenderingContext).getExtension(
        "WEBGL_lose_context"
      );
      loseContext?.loseContext();
    }

    if (this.svgPreviewImage) {
      this.svgPreviewImage.remove();
      this.svgPreviewImage = null;
    }
    if (this.svgPreviewUrl) {
      URL.revokeObjectURL(this.svgPreviewUrl);
      this.svgPreviewUrl = null;
    }
    this.currentSvgWidth = 0;
    this.currentSvgHeight = 0;

    this.context = null;
    this.renderMode = null;
  }

  loadMapData(mapData: string | MapData) {
    try {
      const parsed = typeof mapData === "string" ? (JSON.parse(mapData) as MapData) : mapData;
      if (parsed !== this.mapData) {
        this.satelliteTextureCache = null;
      }
      this.mapData = parsed;
      this.resetDrawScale();
      return true;
    } catch {
      return false;
    }
  }

  setLayers(layers: MapLayers) {
    this.layers = { ...layers };
  }

  setSatelliteStyle(enabled: boolean) {
    this.useSatelliteStyle = enabled;
    // 触发重新渲染
    if (this.mapData) {
      void this.render();
    }
  }

  async render() {
    if (!this.mapData || !this.renderMode) {
      return;
    }

    this.resetDrawScale();
    this.updateDrawScale(this.mapData.draw_scale);

    const width = this.mapData.image_width;
    const height = this.mapData.image_height;

    if (this.renderMode === "svg") {
      await this.renderToSVG(width, height);
      return;
    }

    this.canvas.style.display = "block";
    this.canvas.width = width;
    this.canvas.height = height;
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;
    this.canvas.style.imageRendering = this.useSatelliteStyle ? "auto" : "crisp-edges";

    if (this.renderMode === "canvas") {
      if (this.useSatelliteStyle && this.hasSatelliteRasterData(this.mapData)) {
        const imageData = await this.rasterizeSatelliteStyleToImageData(width, height);
        (this.context as CanvasRenderingContext2D).putImageData(imageData, 0, 0);
        this.canvas.style.visibility = "visible";
        return;
      }
      this.renderToCanvas2D(this.context as CanvasRenderingContext2D, width, height);
      this.canvas.style.visibility = "visible";
      return;
    }

    if (this.useSatelliteStyle && this.hasSatelliteRasterData(this.mapData)) {
      try {
        if (this.renderMode === "webgpu") {
          await this.renderSatelliteToWebGPU(width, height);
          this.canvas.style.visibility = "visible";
          return;
        }

        this.renderSatelliteToWebGL(width, height);
        this.canvas.style.visibility = "visible";
        return;
      } catch (error) {
        console.error("Satellite GPU renderer failed, falling back to standard compositing.", error);
        const fallbackImageData = await this.rasterizeSatelliteStyleToImageData(width, height);
        if (this.renderMode === "webgpu") {
          await this.copyToWebGPU(fallbackImageData);
        } else {
          this.copyToWebGL(fallbackImageData);
        }
        this.canvas.style.visibility = "visible";
        return;
      }
    }

    const offscreenCanvas = document.createElement("canvas");
    offscreenCanvas.width = width;
    offscreenCanvas.height = height;
    const ctx2d = offscreenCanvas.getContext("2d", { alpha: false });
    if (!ctx2d) {
      throw new Error("Failed to create offscreen canvas");
    }

    this.renderToCanvas2D(ctx2d, width, height);
    const imageData = ctx2d.getImageData(0, 0, width, height);

    if (this.renderMode === "webgpu") {
      await this.copyToWebGPU(imageData);
      this.canvas.style.visibility = "visible";
      return;
    }

    this.copyToWebGL(imageData);
    this.canvas.style.visibility = "visible";
  }

  async buildSVGString() {
    if (!this.mapData) {
      throw new Error("No map data to export");
    }

    // 卫星风格依赖 Rust 导出的 heightmap + land_mask 栅格数据
    if (this.useSatelliteStyle && this.hasSatelliteRasterData(this.mapData)) {
      return generateSatelliteSVG(this.mapData, this.layers, this.getSatelliteExportSvgOptions());
    }

    // 否则使用传统矢量渲染
    return generateSVGString(this.mapData, this.getSvgStyles(), this.layers);
  }

  async exportToPNG() {
    if (this.renderMode === "svg") {
      return this.svgToPNG();
    }

    return this.canvas.toDataURL("image/png");
  }

  private rgbaToStyle(rgba: readonly [number, number, number, number]) {
    const r = Math.round(rgba[0] * 255);
    const g = Math.round(rgba[1] * 255);
    const b = Math.round(rgba[2] * 255);
    return `rgba(${r}, ${g}, ${b}, ${rgba[3]})`;
  }

  private getSlopeStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.10, 0.17, 0.10, 0.12] : this.SLOPE_RGBA;
  }

  private getRiverStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.42, 0.89, 0.98, 0.96] : this.RIVER_RGBA;
  }

  private getRiverGlowStyle(): readonly [number, number, number, number] {
    return [0.36, 0.86, 0.96, this.useSatelliteStyle ? 0.28 : 0];
  }

  private getContourStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.18, 0.26, 0.14, 0.16] : this.CONTOUR_RGBA;
  }

  private getBorderStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.82, 0.90, 0.78, 0.44] : this.BORDER_RGBA;
  }

  private getBorderUnderStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.12, 0.30, 0.24, 0.10] : this.BACKGROUND_RGBA;
  }

  private getCityOuterStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.16, 0.23, 0.18, 0.88] : this.CITY_MARKER_RGBA;
  }

  private getCityInnerStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.97, 0.97, 0.88, 0.96] : this.BACKGROUND_RGBA;
  }

  private getTownStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.94, 0.96, 0.84, 0.92] : this.TOWN_MARKER_RGBA;
  }

  private getLabelFillStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.93, 0.97, 0.96, 0.94] : this.TEXT_RGBA;
  }

  private getLabelStrokeStyle(): readonly [number, number, number, number] {
    return this.useSatelliteStyle ? [0.10, 0.18, 0.25, 0.70] : this.BACKGROUND_RGBA;
  }

  private createCanvas() {
    const canvas = document.createElement("canvas");
    canvas.className = "pointer-events-none block max-w-none max-h-none bg-white";
    canvas.style.imageRendering = "crisp-edges";
    canvas.style.visibility = "hidden";
    this.stage.appendChild(canvas);
    return canvas;
  }

  private replaceCanvas() {
    this.cleanup();
    this.canvas.remove();
    this.canvas = this.createCanvas();
  }

  private async detectAvailableModes() {
    const modes: RenderBackend[] = [];

    if (await this.canUseWebGPU()) {
      modes.push("webgpu");
    }
    if (this.canUseWebGL()) {
      modes.push("webgl");
    }
    if (this.canUseCanvas()) {
      modes.push("canvas");
    }

    modes.push("svg");
    this.availableModes = modes;
  }

  private async canUseWebGPU() {
    try {
      const gpuNavigator = navigator as Navigator & { gpu?: GPU };
      if (!gpuNavigator.gpu) {
        return false;
      }

      const adapter = await gpuNavigator.gpu.requestAdapter();
      return Boolean(adapter);
    } catch {
      return false;
    }
  }

  private canUseWebGL() {
    try {
      return Boolean(
        document.createElement("canvas").getContext("webgl2") ||
          document.createElement("canvas").getContext("webgl")
      );
    } catch {
      return false;
    }
  }

  private canUseCanvas() {
    try {
      return Boolean(document.createElement("canvas").getContext("2d"));
    } catch {
      return false;
    }
  }

  private async initializeMode(mode: RenderBackend) {
    switch (mode) {
      case "webgpu":
        return this.tryInitWebGPU();
      case "webgl":
        return this.tryInitWebGL();
      case "canvas":
        return this.tryInitCanvas();
      case "svg":
        return this.tryInitSVG();
    }
  }

  private async tryInitWebGPU() {
    const gpuNavigator = navigator as Navigator & { gpu?: GPU };
    if (!gpuNavigator.gpu) {
      return false;
    }

    const adapter = await gpuNavigator.gpu.requestAdapter();
    if (!adapter) {
      return false;
    }

    this.device = await adapter.requestDevice();
    const context = this.canvas.getContext("webgpu") as GPUCanvasContext | null;
    if (!context) {
      return false;
    }

    context.configure({
      device: this.device,
      format: gpuNavigator.gpu.getPreferredCanvasFormat(),
      alphaMode: "opaque"
    });

    this.context = context;
    this.renderMode = "webgpu";
    return true;
  }

  private tryInitWebGL() {
    const context =
      this.canvas.getContext("webgl2", { preserveDrawingBuffer: true }) ||
      this.canvas.getContext("webgl", { preserveDrawingBuffer: true });

    if (!context) {
      return false;
    }

    this.context = context;
    this.renderMode = "webgl";
    return true;
  }

  private tryInitCanvas() {
    const context = this.canvas.getContext("2d", { alpha: false });
    if (!context) {
      return false;
    }

    this.context = context;
    this.renderMode = "canvas";
    return true;
  }

  private tryInitSVG() {
    this.context = null;
    this.renderMode = "svg";
    return true;
  }

  private resetDrawScale() {
    this.SLOPE_LINE_WIDTH = 1;
    this.RIVER_LINE_WIDTH = 2.5;
    this.CONTOUR_LINE_WIDTH = 1.5;
    this.BORDER_LINE_WIDTH = 6;
    this.BORDER_DASH_PATTERN = [3, 4];
    this.CITY_MARKER_OUTER_RADIUS = 10;
    this.CITY_MARKER_INNER_RADIUS = 5;
    this.TOWN_MARKER_RADIUS = 5;
  }

  private updateDrawScale(scale: number) {
    this.SLOPE_LINE_WIDTH *= scale;
    this.RIVER_LINE_WIDTH *= scale;
    this.CONTOUR_LINE_WIDTH *= scale;
    this.BORDER_LINE_WIDTH *= scale;
    this.BORDER_DASH_PATTERN = this.BORDER_DASH_PATTERN.map((value) => value * scale);
    this.CITY_MARKER_OUTER_RADIUS *= scale;
    this.CITY_MARKER_INNER_RADIUS *= scale;
    this.TOWN_MARKER_RADIUS *= scale;
  }

  private renderToCanvas2D(context: CanvasRenderingContext2D, width: number, height: number) {
    context.clearRect(0, 0, width, height);
    context.fillStyle = this.rgbaToStyle(this.BACKGROUND_RGBA);
    context.fillRect(0, 0, width, height);
    this.drawConfiguredLayers(context, width, height);
  }

  private drawConfiguredLayers(context: CanvasRenderingContext2D, width: number, height: number) {
    if (this.layers.slope) {
      context.lineWidth = this.SLOPE_LINE_WIDTH * (this.useSatelliteStyle ? 0.8 : 1);
      context.strokeStyle = this.rgbaToStyle(this.getSlopeStyle());
      this.drawSegments(this.mapData?.slope ?? [], context, width, height);
    }

    if (this.layers.border) {
      context.lineWidth = this.BORDER_LINE_WIDTH;
      context.strokeStyle = this.rgbaToStyle(this.getBorderUnderStyle());
      this.drawPaths(this.mapData?.territory ?? [], context, width, height);

      context.lineWidth = this.BORDER_LINE_WIDTH * (this.useSatelliteStyle ? 0.52 : 1);
      context.setLineDash(this.BORDER_DASH_PATTERN);
      context.lineCap = "butt";
      context.lineJoin = "bevel";
      context.strokeStyle = this.rgbaToStyle(this.getBorderStyle());
      this.drawPaths(this.mapData?.territory ?? [], context, width, height);
      context.setLineDash([]);
    }

    if (this.layers.river) {
      if (this.useSatelliteStyle) {
        context.lineWidth = this.RIVER_LINE_WIDTH * 1.8;
        context.strokeStyle = this.rgbaToStyle(this.getRiverGlowStyle());
        this.drawPaths(this.mapData?.river ?? [], context, width, height);
      }
      context.lineWidth = this.RIVER_LINE_WIDTH * (this.useSatelliteStyle ? 0.92 : 1);
      context.strokeStyle = this.rgbaToStyle(this.getRiverStyle());
      this.drawPaths(this.mapData?.river ?? [], context, width, height);
    }

    if (this.layers.contour) {
      context.lineWidth = this.CONTOUR_LINE_WIDTH * (this.useSatelliteStyle ? 0.85 : 1);
      context.strokeStyle = this.rgbaToStyle(this.getContourStyle());
      context.lineCap = "round";
      context.lineJoin = "round";
      this.drawPaths(this.mapData?.contour ?? [], context, width, height);
    }

    if (this.layers.city) {
      this.drawCities(this.mapData?.city ?? [], context, width, height);
    }
    if (this.layers.town) {
      this.drawTowns(this.mapData?.town ?? [], context, width, height);
    }
    if (this.layers.label) {
      this.drawLabels(this.mapData?.label ?? [], context, width, height);
    }
  }

  private buildOverlayImageData(width: number, height: number) {
    const overlayCanvas = document.createElement("canvas");
    overlayCanvas.width = width;
    overlayCanvas.height = height;
    const overlayContext = overlayCanvas.getContext("2d", { alpha: true });

    if (!overlayContext) {
      throw new Error("Failed to create overlay canvas");
    }

    overlayContext.clearRect(0, 0, width, height);
    this.drawConfiguredLayers(overlayContext, width, height);
    return overlayContext.getImageData(0, 0, width, height);
  }

  private drawPaths(data: number[][], context: CanvasRenderingContext2D, width: number, height: number) {
    for (const path of data) {
      context.beginPath();
      for (let index = 0; index < path.length; index += 2) {
        const x = path[index] * width;
        const y = height - path[index + 1] * height;

        if (index === 0) {
          context.moveTo(x, y);
        } else {
          context.lineTo(x, y);
        }
      }
      context.stroke();
    }
  }

  private drawSegments(data: number[], context: CanvasRenderingContext2D, width: number, height: number) {
    context.beginPath();
    for (let index = 0; index < data.length; index += 4) {
      const x1 = data[index] * width;
      const y1 = height - data[index + 1] * height;
      const x2 = data[index + 2] * width;
      const y2 = height - data[index + 3] * height;
      context.moveTo(x1, y1);
      context.lineTo(x2, y2);
    }
    context.stroke();
  }

  private drawCities(data: number[], context: CanvasRenderingContext2D, width: number, height: number) {
    for (let index = 0; index < data.length; index += 2) {
      const x = data[index] * width;
      const y = height - data[index + 1] * height;
      context.fillStyle = this.rgbaToStyle(this.getCityOuterStyle());
      context.beginPath();
      context.arc(x, y, this.CITY_MARKER_OUTER_RADIUS, 0, Math.PI * 2);
      context.fill();

      context.fillStyle = this.rgbaToStyle(this.getCityInnerStyle());
      context.beginPath();
      context.arc(x, y, this.CITY_MARKER_INNER_RADIUS, 0, Math.PI * 2);
      context.fill();
    }
  }

  private drawTowns(data: number[], context: CanvasRenderingContext2D, width: number, height: number) {
    context.fillStyle = this.rgbaToStyle(this.getTownStyle());
    for (let index = 0; index < data.length; index += 2) {
      const x = data[index] * width;
      const y = height - data[index + 1] * height;
      context.beginPath();
      context.arc(x, y, this.TOWN_MARKER_RADIUS, 0, Math.PI * 2);
      context.fill();
    }
  }

  private drawLabels(data: MapData["label"], context: CanvasRenderingContext2D, width: number, height: number) {
    for (const label of data) {
      context.font = `${label.fontsize}px ${label.fontface}`;
      context.textAlign = "start";
      context.textBaseline = "alphabetic";
      context.lineJoin = "round";
      const x = label.position[0] * width;
      const y = (1 - label.position[1]) * height;
      context.strokeStyle = this.rgbaToStyle(this.getLabelStrokeStyle());
      context.lineWidth = this.useSatelliteStyle ? 4 : 6;
      context.strokeText(label.text, x, y);
      context.fillStyle = this.rgbaToStyle(this.getLabelFillStyle());
      context.fillText(label.text, x, y);
    }
  }

  private async renderToSVG(width: number, height: number) {
    const previewVersion = ++this.svgPreviewVersion;
    this.canvas.style.display = "none";
    this.canvas.width = width;
    this.canvas.height = height;

    this.svgPreviewImage?.remove();
    if (this.svgPreviewUrl) {
      URL.revokeObjectURL(this.svgPreviewUrl);
      this.svgPreviewUrl = null;
    }

    const svgString = this.useSatelliteStyle && this.hasSatelliteRasterData(this.mapData)
      ? await generateSatelliteSVG(this.mapData, this.layers, this.getSatellitePreviewSvgOptions())
      : generateSVGString(this.mapData as MapData, this.getSvgStyles(), this.layers);

    this.currentSvgWidth = width;
    this.currentSvgHeight = height;

    const blob = new Blob([svgString], { type: "image/svg+xml;charset=utf-8" });
    const url = URL.createObjectURL(blob);

    try {
      const image = await this.loadSvgPreviewImage(url, width, height);
      if (previewVersion !== this.svgPreviewVersion) {
        URL.revokeObjectURL(url);
        return;
      }
      this.svgPreviewUrl = url;
      this.svgPreviewImage = image;
      this.stage.appendChild(image);
    } catch (error) {
      URL.revokeObjectURL(url);
      throw error;
    }
  }

  private getSatellitePreviewSvgOptions() {
    return {
      maxEmbeddedImageSize: 1024,
      jpegQuality: 70
    } as const;
  }

  private getSatelliteExportSvgOptions() {
    return {
      maxEmbeddedImageSize: 1600,
      jpegQuality: 82
    } as const;
  }

  private loadSvgPreviewImage(url: string, width: number, height: number) {
    return new Promise<HTMLImageElement>((resolve, reject) => {
      const image = new Image();
      image.decoding = "async";
      image.draggable = false;
      image.width = width;
      image.height = height;
      image.className = "block bg-white select-none";
      image.onload = () => resolve(image);
      image.onerror = () => reject(new Error("Failed to load SVG preview image"));
      image.src = url;
    });
  }

  private getSvgStyles() {
    return {
      BACKGROUND_RGBA: this.rgbaToStyle(this.BACKGROUND_RGBA),
      SLOPE_RGBA: this.rgbaToStyle(this.SLOPE_RGBA),
      RIVER_RGBA: this.rgbaToStyle(this.RIVER_RGBA),
      CONTOUR_RGBA: this.rgbaToStyle(this.CONTOUR_RGBA),
      BORDER_RGBA: this.rgbaToStyle(this.BORDER_RGBA),
      CITY_MARKER_RGBA: this.rgbaToStyle(this.CITY_MARKER_RGBA),
      TOWN_MARKER_RGBA: this.rgbaToStyle(this.TOWN_MARKER_RGBA),
      TEXT_RGBA: this.rgbaToStyle(this.TEXT_RGBA),
      SLOPE_LINE_WIDTH: this.SLOPE_LINE_WIDTH,
      RIVER_LINE_WIDTH: this.RIVER_LINE_WIDTH,
      CONTOUR_LINE_WIDTH: this.CONTOUR_LINE_WIDTH,
      BORDER_LINE_WIDTH: this.BORDER_LINE_WIDTH,
      BORDER_DASH_PATTERN: this.BORDER_DASH_PATTERN.join(","),
      CITY_MARKER_OUTER_RADIUS: this.CITY_MARKER_OUTER_RADIUS,
      CITY_MARKER_INNER_RADIUS: this.CITY_MARKER_INNER_RADIUS,
      TOWN_MARKER_RADIUS: this.TOWN_MARKER_RADIUS
    };
  }

  private hasSatelliteRasterData(
    mapData: MapData | null
  ): mapData is MapData & {
    heightmap: NonNullable<MapData["heightmap"]>;
    land_mask: NonNullable<MapData["land_mask"]>;
  } {
    return Boolean(mapData?.heightmap && mapData.land_mask);
  }

  private async rasterizeSatelliteStyleToImageData(width: number, height: number) {
    if (!this.mapData || !this.hasSatelliteRasterData(this.mapData)) {
      throw new Error("Satellite raster data is not available");
    }

    const svgString = await generateSatelliteSVG(
      this.mapData,
      this.layers,
      this.getSatellitePreviewSvgOptions()
    );
    const blob = new Blob([svgString], { type: "image/svg+xml;charset=utf-8" });
    const url = URL.createObjectURL(blob);

    try {
      const image = await new Promise<HTMLImageElement>((resolve, reject) => {
        const element = new Image();
        element.onload = () => resolve(element);
        element.onerror = () => reject(new Error("Failed to load satellite SVG fallback image"));
        element.src = url;
      });

      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      const context = canvas.getContext("2d", { alpha: false });
      if (!context) {
        throw new Error("Failed to create fallback raster canvas");
      }

      context.drawImage(image, 0, 0, width, height);
      return context.getImageData(0, 0, width, height);
    } finally {
      URL.revokeObjectURL(url);
    }
  }

  private getSatelliteTextures(
    mapData: MapData & {
      heightmap: NonNullable<MapData["heightmap"]>;
      land_mask: NonNullable<MapData["land_mask"]>;
    }
  ) {
    if (this.satelliteTextureCache?.mapData === mapData) {
      return this.satelliteTextureCache;
    }

    const height = this.encodeRasterTexture(
      mapData.heightmap.data,
      mapData.heightmap.width,
      mapData.heightmap.height,
      (value) => Math.round(this.clamp01(value) * 255)
    );
    const land =
      mapData.land_polygons && mapData.land_polygons.length > 0
        ? this.encodeLandPolygonTexture(
            mapData.land_polygons,
            mapData.image_width,
            mapData.image_height
          )
        : this.encodeRasterTexture(
            mapData.land_mask.data,
            mapData.land_mask.width,
            mapData.land_mask.height,
            (value) => (value > 0 ? 255 : 0)
          );
    const flux = mapData.flux_map
      ? this.encodeRasterTexture(
          mapData.flux_map.data,
          mapData.flux_map.width,
          mapData.flux_map.height,
          (value) => Math.round(this.clamp01(value) * 255)
        )
      : this.encodeSolidTexture(0);

    const bundle: SatelliteTextureBundle = {
      mapData,
      height,
      land,
      flux
    };
    this.satelliteTextureCache = bundle;
    return bundle;
  }

  private clamp01(value: number) {
    return Math.max(0, Math.min(1, value));
  }

  private encodeSolidTexture(value: number): EncodedRasterTexture {
    const bytes = new Uint8Array([value, value, value, 255]);
    return {
      width: 1,
      height: 1,
      texelSize: [1, 1],
      data: bytes
    };
  }

  private encodeRasterTexture(
    data: number[],
    width: number,
    height: number,
    encode: (value: number) => number
  ): EncodedRasterTexture {
    const bytes = new Uint8Array(width * height * 4);

    for (let y = 0; y < height; y += 1) {
      const sourceRow = height - 1 - y;
      for (let x = 0; x < width; x += 1) {
        const sourceIndex = sourceRow * width + x;
        const targetIndex = (y * width + x) * 4;
        const encoded = encode(data[sourceIndex] ?? 0);
        bytes[targetIndex] = encoded;
        bytes[targetIndex + 1] = encoded;
        bytes[targetIndex + 2] = encoded;
        bytes[targetIndex + 3] = 255;
      }
    }

    return {
      width,
      height,
      texelSize: [1 / width, 1 / height],
      data: bytes
    };
  }

  private encodeLandPolygonTexture(
    polygons: number[][],
    width: number,
    height: number
  ): EncodedRasterTexture {
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext("2d", { alpha: true });

    if (!context) {
      throw new Error("Failed to create precise land-mask canvas");
    }

    context.clearRect(0, 0, width, height);
    context.fillStyle = "#ffffff";

    for (const polygon of polygons) {
      if (polygon.length < 6) {
        continue;
      }

      context.beginPath();
      context.moveTo(polygon[0] * width, height - polygon[1] * height);
      for (let index = 2; index < polygon.length; index += 2) {
        context.lineTo(polygon[index] * width, height - polygon[index + 1] * height);
      }
      context.closePath();
      context.fill();
    }

    const source = context.getImageData(0, 0, width, height).data;
    const bytes = new Uint8Array(width * height * 4);
    for (let index = 0; index < source.length; index += 4) {
      const coverage = source[index + 3] ?? 0;
      bytes[index] = coverage;
      bytes[index + 1] = coverage;
      bytes[index + 2] = coverage;
      bytes[index + 3] = 255;
    }

    return {
      width,
      height,
      texelSize: [1 / width, 1 / height],
      data: bytes
    };
  }

  private async renderSatelliteToWebGPU(width: number, height: number) {
    const gpuNavigator = navigator as Navigator & { gpu?: GPU };
    if (!this.device || !this.context || !gpuNavigator.gpu || !this.hasSatelliteRasterData(this.mapData)) {
      throw new Error("WebGPU satellite renderer is not ready");
    }

    const mapData = this.mapData;
    const textures = this.getSatelliteTextures(mapData);
    const overlay = this.buildOverlayImageData(width, height);
    const context = this.context as GPUCanvasContext;

    this.device.pushErrorScope("validation");
    this.device.pushErrorScope("internal");

    try {
      context.configure({
        device: this.device,
        format: gpuNavigator.gpu.getPreferredCanvasFormat(),
        alphaMode: "opaque"
      });

      const heightTexture = this.createWebGPUTextureFromBytes(
        this.device,
        textures.height,
        "Satellite Height Texture"
      );
      const landTexture = this.createWebGPUTextureFromBytes(
        this.device,
        textures.land,
        "Satellite Land Texture"
      );
      const fluxTexture = this.createWebGPUTextureFromBytes(
        this.device,
        textures.flux,
        "Satellite Flux Texture"
      );
      const overlayTexture = this.device.createTexture({
        label: "Satellite Overlay Texture",
        size: [overlay.width, overlay.height],
        format: "rgba8unorm",
        usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST
      });

      this.device.queue.writeTexture(
        { texture: overlayTexture },
        overlay.data,
        { bytesPerRow: overlay.width * 4 },
        { width: overlay.width, height: overlay.height }
      );

      const uniformBuffer = this.device.createBuffer({
        label: "Satellite Terrain Uniforms",
        size: 32,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
      });
      this.device.queue.writeBuffer(
        uniformBuffer,
        0,
        new Float32Array([
          textures.height.texelSize[0],
          textures.height.texelSize[1],
          textures.land.texelSize[0],
          textures.land.texelSize[1],
          mapData.flux_map ? 1 : 0,
          0,
          0,
          0
        ])
      );

      const sampler = this.device.createSampler({
        magFilter: "linear",
        minFilter: "linear",
        addressModeU: "clamp-to-edge",
        addressModeV: "clamp-to-edge"
      });

      const shaderModule = this.device.createShaderModule({
      label: "Satellite Terrain Shader",
      code: `
        struct VertexOutput {
          @builtin(position) position: vec4f,
          @location(0) uv: vec2f,
        };

        struct TerrainUniforms {
          height_texel: vec2f,
          land_texel: vec2f,
          has_flux: f32,
          pad0: f32,
        };

        @group(0) @binding(0) var height_tex: texture_2d<f32>;
        @group(0) @binding(1) var land_tex: texture_2d<f32>;
        @group(0) @binding(2) var flux_tex: texture_2d<f32>;
        @group(0) @binding(3) var overlay_tex: texture_2d<f32>;
        @group(0) @binding(4) var terrain_sampler: sampler;
        @group(0) @binding(5) var<uniform> uniforms: TerrainUniforms;

        @vertex
        fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
          var pos = array<vec2f, 6>(
            vec2f(-1.0, -1.0), vec2f(1.0, -1.0), vec2f(-1.0, 1.0),
            vec2f(-1.0, 1.0), vec2f(1.0, -1.0), vec2f(1.0, 1.0)
          );
          var uv = array<vec2f, 6>(
            vec2f(0.0, 1.0), vec2f(1.0, 1.0), vec2f(0.0, 0.0),
            vec2f(0.0, 0.0), vec2f(1.0, 1.0), vec2f(1.0, 0.0)
          );

          var output: VertexOutput;
          output.position = vec4f(pos[idx], 0.0, 1.0);
          output.uv = uv[idx];
          return output;
        }

        fn saturate(value: f32) -> f32 {
          return clamp(value, 0.0, 1.0);
        }

        fn sample_height(uv: vec2f) -> f32 {
          return textureSample(height_tex, terrain_sampler, clamp(uv, vec2f(0.001), vec2f(0.999))).r;
        }

        fn sample_land(uv: vec2f) -> f32 {
          return textureSample(land_tex, terrain_sampler, clamp(uv, vec2f(0.001), vec2f(0.999))).r;
        }

        fn sample_flux(uv: vec2f) -> f32 {
          if (uniforms.has_flux < 0.5) {
            return 0.0;
          }
          return textureSample(flux_tex, terrain_sampler, clamp(uv, vec2f(0.001), vec2f(0.999))).r;
        }

        fn hash21(p: vec2f) -> f32 {
          return fract(sin(dot(p, vec2f(127.1, 311.7))) * 43758.5453123);
        }

        fn biome_noise(uv: vec2f) -> f32 {
          let warp = vec2f(
            hash21(uv * vec2f(19.0, 13.0)) - 0.5,
            hash21(uv * vec2f(17.0, 23.0)) - 0.5
          ) * 0.42;
          let warped_uv = uv + warp;
          let broad = hash21(warped_uv * vec2f(37.0, 29.0));
          let medium = hash21(warped_uv * vec2f(91.0, 67.0));
          let detail = hash21(warped_uv * vec2f(221.0, 173.0));
          return broad * 0.56 + medium * 0.30 + detail * 0.14;
        }

        fn terrace_height(height: f32, strength: f32) -> f32 {
          let steps = 7.0;
          let terraced = floor(height * steps) / steps;
          return mix(height, terraced, strength);
        }

        fn terrain_color(
          height: f32,
          slope: f32,
          moisture: f32,
          latitude: f32,
          rugged: f32,
          inland: f32,
          biome_mask: f32
        ) -> vec3f {
          let beach = vec3f(0.88, 0.83, 0.60);
          let meadow = vec3f(0.62, 0.77, 0.30);
          let fertile = vec3f(0.38, 0.62, 0.24);
          let forest = vec3f(0.20, 0.42, 0.17);
          let grove = vec3f(0.30, 0.54, 0.20);
          let plateau = vec3f(0.62, 0.58, 0.31);
          let dryland = vec3f(0.76, 0.67, 0.35);
          let prairie = vec3f(0.74, 0.78, 0.32);
          let rock = vec3f(0.52, 0.53, 0.50);
          let cliff = vec3f(0.46, 0.47, 0.52);
          let tundra = vec3f(0.70, 0.76, 0.66);
          let snow = vec3f(0.97, 0.98, 1.0);
          let coolness = smoothstep(0.68, 1.0, latitude);
          let dryness = inland * (1.0 - moisture * 0.74);
          let lowland = mix(meadow, fertile, moisture);
          let lush_patch = smoothstep(0.52, 0.82, biome_mask + moisture * 0.24 - inland * 0.08);
          let dry_patch = smoothstep(0.56, 0.86, (1.0 - biome_mask) + dryness * 0.28);
          let lowland_patch = mix(lowland, prairie, dry_patch * (1.0 - moisture) * 0.72);
          let wooded = mix(lowland_patch, forest, smoothstep(0.22, 0.76, moisture + inland * 0.18));
          let wooded_patch = mix(wooded, grove, lush_patch * 0.46);
          let warm_upland = mix(plateau, dryland, dryness * 0.82);
          let exposed_rock = mix(rock, cliff, rugged);

          let terrace_strength = smoothstep(0.28, 0.74, height) * (0.18 + rugged * 0.16);
          let terraced_height = terrace_height(height, terrace_strength);
          var base = mix(beach, lowland_patch, smoothstep(0.015, 0.06, terraced_height));
          base = mix(base, wooded_patch, smoothstep(0.06, 0.22, terraced_height));
          base = mix(base, warm_upland, smoothstep(0.20, 0.48, terraced_height) * (0.50 + dryness * 0.65));
          base = mix(base, tundra, smoothstep(0.42, 0.70, terraced_height) * coolness * 0.36);
          base = mix(base, exposed_rock, smoothstep(0.46, 0.76, terraced_height + slope * 0.14));
          base = mix(base, snow, smoothstep(0.76 - coolness * 0.10, 0.94, terraced_height + rugged * 0.05));
          return base;
        }

        @fragment
        fn fs_main(input: VertexOutput) -> @location(0) vec4f {
          let uv = input.uv;
          let land = sample_land(uv);
          let h = sample_height(uv);
          let h_l = sample_height(uv - vec2f(uniforms.height_texel.x, 0.0));
          let h_r = sample_height(uv + vec2f(uniforms.height_texel.x, 0.0));
          let h_u = sample_height(uv - vec2f(0.0, uniforms.height_texel.y));
          let h_d = sample_height(uv + vec2f(0.0, uniforms.height_texel.y));
          let far_offset = uniforms.height_texel * 4.0;
          let h_l_far = sample_height(uv - vec2f(far_offset.x, 0.0));
          let h_r_far = sample_height(uv + vec2f(far_offset.x, 0.0));
          let h_u_far = sample_height(uv - vec2f(0.0, far_offset.y));
          let h_d_far = sample_height(uv + vec2f(0.0, far_offset.y));
          let avg_neighbors = (h_l + h_r + h_u + h_d) * 0.25;
          let avg_neighbors_far = (h_l_far + h_r_far + h_u_far + h_d_far) * 0.25;
          let grad = vec2f(h_r - h_l, h_d - h_u);
          let grad_far = vec2f(h_r_far - h_l_far, h_d_far - h_u_far);
          let slope_near = saturate(length(grad) * 7.8);
          let slope_far = saturate(length(grad_far) * 4.6);
          let slope = saturate(max(slope_near, slope_far * 0.88));
          let curvature_near = clamp((avg_neighbors - h) * 7.0, -1.0, 1.0);
          let curvature_far = clamp((avg_neighbors_far - h) * 5.4, -1.0, 1.0);
          let normal_near = normalize(vec3f((h_l - h_r) * 6.2, (h_u - h_d) * 6.2, 1.0));
          let normal_far = normalize(vec3f((h_l_far - h_r_far) * 11.0, (h_u_far - h_d_far) * 11.0, 1.0));
          let normal = normalize(mix(normal_far, normal_near, 0.64 + slope_near * 0.18));

          let key_light = normalize(vec3f(-0.58, 0.44, 0.73));
          let fill_light = normalize(vec3f(0.34, -0.16, 0.40));
          let rim_light = normalize(vec3f(0.08, 0.78, 0.24));
          let key = max(dot(normal, key_light), 0.0);
          let fill = max(dot(normal, fill_light), 0.0);
          let rim = max(dot(normal, rim_light), 0.0);
          let hillshade = saturate(0.34 + key * 0.78 + fill * 0.24 + rim * 0.10);
          let ambient_occlusion = clamp(1.0 - curvature_near * 0.16 - curvature_far * 0.18 - slope_near * 0.05, 0.72, 1.08);
          let ridge = saturate((-curvature_near * 0.78) + slope_near * 0.28 + slope_far * 0.24 - 0.16);
          let valley = saturate(curvature_near * 0.72 + curvature_far * 0.44 - 0.06);

          let nearby_land = (
            sample_land(uv + vec2f(uniforms.land_texel.x * 2.0, 0.0)) +
            sample_land(uv - vec2f(uniforms.land_texel.x * 2.0, 0.0)) +
            sample_land(uv + vec2f(0.0, uniforms.land_texel.y * 2.0)) +
            sample_land(uv - vec2f(0.0, uniforms.land_texel.y * 2.0)) +
            sample_land(uv + uniforms.land_texel * 2.0) +
            sample_land(uv - uniforms.land_texel * 2.0) +
            sample_land(uv + vec2f(uniforms.land_texel.x * 2.0, -uniforms.land_texel.y * 2.0)) +
            sample_land(uv + vec2f(-uniforms.land_texel.x * 2.0, uniforms.land_texel.y * 2.0))
          ) / 8.0;
          let regional_land = (
            sample_land(uv + vec2f(uniforms.land_texel.x * 18.0, 0.0)) +
            sample_land(uv - vec2f(uniforms.land_texel.x * 18.0, 0.0)) +
            sample_land(uv + vec2f(0.0, uniforms.land_texel.y * 18.0)) +
            sample_land(uv - vec2f(0.0, uniforms.land_texel.y * 18.0)) +
            sample_land(uv + vec2f(uniforms.land_texel.x * 14.0, uniforms.land_texel.y * 14.0)) +
            sample_land(uv + vec2f(-uniforms.land_texel.x * 14.0, uniforms.land_texel.y * 14.0)) +
            sample_land(uv + vec2f(uniforms.land_texel.x * 14.0, -uniforms.land_texel.y * 14.0)) +
            sample_land(uv + vec2f(-uniforms.land_texel.x * 14.0, -uniforms.land_texel.y * 14.0))
          ) / 8.0;
          let coast = saturate(abs(nearby_land - land) * 1.45);
          let river = smoothstep(0.06, 0.65, sample_flux(uv)) * land;
          let latitude = 1.0 - uv.y;
          let inland = saturate((regional_land - 0.35) * 1.45) * land;
          let moisture = saturate(river * 0.74 + coast * 0.36 + (1.0 - slope_near) * 0.10 + latitude * 0.05 + (1.0 - h) * 0.08 - inland * 0.10);
          let biome_mask = biome_noise(uv * vec2f(5.2, 4.8) + vec2f(inland * 0.9, latitude * 0.35));
          let macro_variation = biome_noise(uv * vec2f(1.6, 1.3) + vec2f(3.1, 1.7));
          let noise = (hash21(uv * vec2f(1531.0, 977.0)) - 0.5) * 0.08 +
            (hash21(uv * vec2f(5211.0, 4099.0)) - 0.5) * 0.04 +
            (hash21(uv * vec2f(11017.0, 9013.0)) - 0.5) * 0.02;
          let overlay = textureSample(overlay_tex, terrain_sampler, uv);

          var color = terrain_color(h, slope, moisture, latitude, ridge, inland, biome_mask);
          let land_coast_glow = coast * smoothstep(0.02, 0.10, h) * (1.0 - inland * 0.5);
          color = color * (hillshade * ambient_occlusion + noise * 0.72);
          color = mix(color, color * vec3f(1.04, 1.02, 0.96), smoothstep(0.58, 0.92, macro_variation) * 0.10);
          color = mix(color, color * vec3f(0.94, 1.02, 1.04), smoothstep(0.08, 0.34, macro_variation) * 0.08);
          color = mix(color, color * vec3f(0.84, 0.92, 0.98), valley * 0.10);
          color = mix(color, mix(color, vec3f(0.95, 0.96, 0.86), 0.34), ridge * 0.12);
          color = mix(color, vec3f(0.35, 0.82, 0.95), river * 0.44);
          color = mix(color, vec3f(0.91, 0.89, 0.68), land_coast_glow * 0.46);
          color = mix(color, vec3f(0.79, 0.90, 0.54), smoothstep(0.38, 0.78, biome_mask) * moisture * 0.10);
          color = mix(color, vec3f(0.88, 0.95, 0.98), smoothstep(0.74, 0.98, h) * 0.06);

          if (land < 0.5) {
            let ocean_noise = (hash21(uv * vec2f(311.0, 587.0)) - 0.5) * 0.04;
            let sparkle = (hash21(uv * vec2f(96.0, 141.0)) - 0.5) * 0.05;
            let deep_ocean = vec3f(0.05, 0.27, 0.55);
            let shelf_ocean = vec3f(0.07, 0.56, 0.84);
            let lagoon = vec3f(0.34, 0.88, 0.94);
            let foam = vec3f(0.78, 0.98, 0.98);
            let radial = distance(uv, vec2f(0.52, 0.48));
            var ocean = mix(deep_ocean, shelf_ocean, pow(coast, 0.78));
            ocean = mix(ocean, lagoon, coast * coast * 0.58);
            let shallow_band = smoothstep(0.04, 0.72, coast);
            let ocean_color = ocean * (1.0 - min(0.14, radial * 0.18)) + foam * shallow_band * 0.12 + ocean_noise + sparkle;
            return vec4f(mix(ocean_color, overlay.rgb, overlay.a), 1.0);
          }

          return vec4f(mix(color, overlay.rgb, overlay.a), 1.0);
        }
      `
      });

      const pipeline = this.device.createRenderPipeline({
      label: "Satellite Terrain Pipeline",
      layout: "auto",
      vertex: { module: shaderModule, entryPoint: "vs_main" },
      fragment: {
        module: shaderModule,
        entryPoint: "fs_main",
        targets: [{ format: gpuNavigator.gpu.getPreferredCanvasFormat() }]
      },
      primitive: { topology: "triangle-list" }
      });

      const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: heightTexture.createView() },
        { binding: 1, resource: landTexture.createView() },
        { binding: 2, resource: fluxTexture.createView() },
        { binding: 3, resource: overlayTexture.createView() },
        { binding: 4, resource: sampler },
        { binding: 5, resource: { buffer: uniformBuffer } }
      ]
      });

      const commandEncoder = this.device.createCommandEncoder({ label: "Satellite Terrain Encoder" });
      const passEncoder = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: context.getCurrentTexture().createView(),
          loadOp: "clear",
          storeOp: "store",
          clearValue: { r: 0.04, g: 0.14, b: 0.25, a: 1 }
        }
      ]
      });

      passEncoder.setPipeline(pipeline);
      passEncoder.setBindGroup(0, bindGroup);
      passEncoder.draw(6);
      passEncoder.end();

      this.device.queue.submit([commandEncoder.finish()]);
      await this.device.queue.onSubmittedWorkDone();
    } finally {
      const internalError = await this.device.popErrorScope();
      const validationError = await this.device.popErrorScope();
      if (internalError) {
        throw new Error(`WebGPU internal error: ${internalError.message}`);
      }
      if (validationError) {
        throw new Error(`WebGPU validation error: ${validationError.message}`);
      }
    }
  }

  private createWebGPUTextureFromBytes(
    device: GPUDevice,
    texture: EncodedRasterTexture,
    label: string
  ) {
    const gpuTexture = device.createTexture({
      label,
      size: [texture.width, texture.height],
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST
    });

    device.queue.writeTexture(
      { texture: gpuTexture },
      texture.data,
      { bytesPerRow: texture.width * 4 },
      { width: texture.width, height: texture.height }
    );

    return gpuTexture;
  }

  private renderSatelliteToWebGL(width: number, height: number) {
    if (!this.hasSatelliteRasterData(this.mapData)) {
      throw new Error("WebGL satellite renderer is not ready");
    }

    const gl = this.context as WebGLRenderingContext | WebGL2RenderingContext;
    const textures = this.getSatelliteTextures(this.mapData);
    const overlay = this.buildOverlayImageData(width, height);

    gl.viewport(0, 0, width, height);

    const vertexShader = this.createShader(
      gl,
      gl.VERTEX_SHADER,
      `
        attribute vec2 a_position;
        attribute vec2 a_texCoord;
        varying vec2 v_texCoord;
        void main() {
          gl_Position = vec4(a_position, 0.0, 1.0);
          v_texCoord = a_texCoord;
        }
      `
    );

    const fragmentShader = this.createShader(
      gl,
      gl.FRAGMENT_SHADER,
      `
        precision highp float;
        varying vec2 v_texCoord;
        uniform sampler2D u_heightTexture;
        uniform sampler2D u_landTexture;
        uniform sampler2D u_fluxTexture;
        uniform sampler2D u_overlayTexture;
        uniform vec2 u_heightTexel;
        uniform vec2 u_landTexel;
        uniform float u_hasFlux;

        float saturate(float value) {
          return clamp(value, 0.0, 1.0);
        }

        float sampleHeight(vec2 uv) {
          return texture2D(u_heightTexture, clamp(uv, vec2(0.001), vec2(0.999))).r;
        }

        float sampleLand(vec2 uv) {
          return texture2D(u_landTexture, clamp(uv, vec2(0.001), vec2(0.999))).r;
        }

        float sampleFlux(vec2 uv) {
          if (u_hasFlux < 0.5) {
            return 0.0;
          }
          return texture2D(u_fluxTexture, clamp(uv, vec2(0.001), vec2(0.999))).r;
        }

        float hash21(vec2 p) {
          return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
        }

        vec3 terrainColor(float height, float slope, float moisture, float latitude, float rugged, float inland) {
          vec3 beach = vec3(0.84, 0.78, 0.62);
          vec3 lowDry = vec3(0.76, 0.66, 0.40);
          vec3 lowWet = vec3(0.25, 0.36, 0.18);
          vec3 midDry = vec3(0.64, 0.57, 0.34);
          vec3 midWet = vec3(0.21, 0.31, 0.18);
          vec3 highland = vec3(0.49, 0.46, 0.28);
          vec3 rock = vec3(0.46, 0.44, 0.43);
          vec3 cliff = vec3(0.38, 0.40, 0.44);
          vec3 alpine = vec3(0.64, 0.66, 0.69);
          vec3 snow = vec3(0.96, 0.97, 0.99);
          float dryness = smoothstep(0.24, 0.92, 1.0 - latitude);
          float continentalDryness = inland * (1.0 - moisture * 0.72);
          vec3 lushLow = mix(lowDry, lowWet, moisture * (1.0 - inland * 0.38));
          vec3 lushMid = mix(midDry, midWet, moisture * 0.82);
          vec3 warmHighland = mix(highland, vec3(0.60, 0.55, 0.33), dryness * 0.30 + continentalDryness * 0.18);
          vec3 exposedRock = mix(rock, cliff, rugged);

          vec3 base = mix(beach, lushLow, smoothstep(0.02, 0.07, height));
          base = mix(base, vec3(0.79, 0.69, 0.43), smoothstep(0.08, 0.34, height) * continentalDryness * 0.55);
          base = mix(base, lushMid, smoothstep(0.08, 0.26, height));
          base = mix(base, warmHighland, smoothstep(0.26, 0.48, height));
          base = mix(base, exposedRock, smoothstep(0.42, 0.70, height + slope * 0.16));
          base = mix(base, alpine, smoothstep(0.68, 0.84, height + rugged * 0.04));
          float snowline = 0.82 - latitude * 0.05 + moisture * 0.02;
          base = mix(base, snow, smoothstep(snowline, min(0.97, snowline + 0.12), height + slope * 0.07));
          return base;
        }

        void main() {
          vec2 uv = v_texCoord;
          float land = sampleLand(uv);
          float h = sampleHeight(uv);
          float hL = sampleHeight(uv - vec2(u_heightTexel.x, 0.0));
          float hR = sampleHeight(uv + vec2(u_heightTexel.x, 0.0));
          float hU = sampleHeight(uv - vec2(0.0, u_heightTexel.y));
          float hD = sampleHeight(uv + vec2(0.0, u_heightTexel.y));
          vec2 farOffset = u_heightTexel * 4.0;
          float hLFar = sampleHeight(uv - vec2(farOffset.x, 0.0));
          float hRFar = sampleHeight(uv + vec2(farOffset.x, 0.0));
          float hUFar = sampleHeight(uv - vec2(0.0, farOffset.y));
          float hDFar = sampleHeight(uv + vec2(0.0, farOffset.y));
          float avgNeighbors = (hL + hR + hU + hD) * 0.25;
          float avgNeighborsFar = (hLFar + hRFar + hUFar + hDFar) * 0.25;
          vec2 grad = vec2(hR - hL, hD - hU);
          vec2 gradFar = vec2(hRFar - hLFar, hDFar - hUFar);
          float slopeNear = saturate(length(grad) * 7.8);
          float slopeFar = saturate(length(gradFar) * 4.6);
          float slope = saturate(max(slopeNear, slopeFar * 0.88));
          float curvatureNear = clamp((avgNeighbors - h) * 7.0, -1.0, 1.0);
          float curvatureFar = clamp((avgNeighborsFar - h) * 5.4, -1.0, 1.0);
          vec3 normalNear = normalize(vec3((hL - hR) * 6.2, (hU - hD) * 6.2, 1.0));
          vec3 normalFar = normalize(vec3((hLFar - hRFar) * 11.0, (hUFar - hDFar) * 11.0, 1.0));
          vec3 normal = normalize(mix(normalFar, normalNear, 0.64 + slopeNear * 0.18));

          vec3 keyLight = normalize(vec3(-0.58, 0.44, 0.73));
          vec3 fillLight = normalize(vec3(0.34, -0.16, 0.40));
          vec3 rimLight = normalize(vec3(0.08, 0.78, 0.24));
          float key = max(dot(normal, keyLight), 0.0);
          float fill = max(dot(normal, fillLight), 0.0);
          float rim = max(dot(normal, rimLight), 0.0);
          float hillshade = saturate(0.24 + key * 0.92 + fill * 0.24 + rim * 0.14);
          float ambientOcclusion = clamp(1.0 - curvatureNear * 0.18 - curvatureFar * 0.22 - slopeNear * 0.08, 0.62, 1.08);
          float ridge = saturate((-curvatureNear * 0.78) + slopeNear * 0.28 + slopeFar * 0.24 - 0.16);
          float valley = saturate(curvatureNear * 0.72 + curvatureFar * 0.44 - 0.06);

          float nearbyLand = (
            sampleLand(uv + vec2(u_landTexel.x * 2.0, 0.0)) +
            sampleLand(uv - vec2(u_landTexel.x * 2.0, 0.0)) +
            sampleLand(uv + vec2(0.0, u_landTexel.y * 2.0)) +
            sampleLand(uv - vec2(0.0, u_landTexel.y * 2.0)) +
            sampleLand(uv + u_landTexel * 2.0) +
            sampleLand(uv - u_landTexel * 2.0) +
            sampleLand(uv + vec2(u_landTexel.x * 2.0, -u_landTexel.y * 2.0)) +
            sampleLand(uv + vec2(-u_landTexel.x * 2.0, u_landTexel.y * 2.0))
          ) / 8.0;
          float regionalLand = (
            sampleLand(uv + vec2(u_landTexel.x * 18.0, 0.0)) +
            sampleLand(uv - vec2(u_landTexel.x * 18.0, 0.0)) +
            sampleLand(uv + vec2(0.0, u_landTexel.y * 18.0)) +
            sampleLand(uv - vec2(0.0, u_landTexel.y * 18.0)) +
            sampleLand(uv + vec2(u_landTexel.x * 14.0, u_landTexel.y * 14.0)) +
            sampleLand(uv + vec2(-u_landTexel.x * 14.0, u_landTexel.y * 14.0)) +
            sampleLand(uv + vec2(u_landTexel.x * 14.0, -u_landTexel.y * 14.0)) +
            sampleLand(uv + vec2(-u_landTexel.x * 14.0, -u_landTexel.y * 14.0))
          ) / 8.0;
          float coast = saturate(abs(nearbyLand - land) * 1.45);
          float river = smoothstep(0.06, 0.65, sampleFlux(uv)) * land;
          float latitude = 1.0 - uv.y;
          float inland = saturate((regionalLand - 0.35) * 1.45) * land;
          float moisture = saturate(river * 0.82 + coast * 0.30 + (1.0 - slopeNear) * 0.08 + latitude * 0.06 + (1.0 - h) * 0.10 - inland * 0.14);
          float noise = (hash21(uv * vec2(1531.0, 977.0)) - 0.5) * 0.08 +
            (hash21(uv * vec2(5211.0, 4099.0)) - 0.5) * 0.04 +
            (hash21(uv * vec2(11017.0, 9013.0)) - 0.5) * 0.02;

          vec3 color = terrainColor(h, slope, moisture, latitude, ridge, inland);
          color = color * (hillshade * ambientOcclusion + noise);
          color = mix(color, color * vec3(0.76, 0.84, 0.94), valley * 0.16);
          color = mix(color, mix(color, vec3(0.90, 0.88, 0.78), 0.46), ridge * 0.18);
          color = mix(color, vec3(0.29, 0.58, 0.76), river * 0.52);
          color = mix(color, vec3(0.88, 0.84, 0.68), coast * smoothstep(0.01, 0.08, h) * 0.65);
          color = mix(color, vec3(0.83, 0.87, 0.91), smoothstep(0.72, 0.98, h) * 0.08);

          if (land < 0.5) {
            float oceanNoise = (hash21(uv * vec2(311.0, 587.0)) - 0.5) * 0.05;
            float watercolor = (hash21(uv * vec2(94.0, 121.0)) - 0.5) * 0.08;
            vec3 deepOcean = vec3(0.03, 0.15, 0.34);
            vec3 shelfOcean = vec3(0.08, 0.42, 0.70);
            vec3 lagoon = vec3(0.30, 0.79, 0.87);
            vec3 foam = vec3(0.74, 0.92, 0.96);
            float distanceToCenter = distance(uv, vec2(0.52, 0.48));
            vec3 ocean = mix(deepOcean, shelfOcean, pow(coast, 0.72));
            ocean = mix(ocean, lagoon, coast * coast * 0.52);
            vec3 oceanColor = ocean * (1.0 - min(0.22, distanceToCenter * 0.28)) + foam * coast * 0.16 + oceanNoise + watercolor;
            vec4 overlay = texture2D(u_overlayTexture, uv);
            gl_FragColor = vec4(mix(oceanColor, overlay.rgb, overlay.a), 1.0);
            return;
          }

          vec4 overlay = texture2D(u_overlayTexture, uv);
          gl_FragColor = vec4(mix(color, overlay.rgb, overlay.a), 1.0);
        }
      `
    );

    const program = this.createProgram(gl, vertexShader, fragmentShader);
    gl.useProgram(program);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([
        -1, -1, 0, 1,
        1, -1, 1, 1,
        -1, 1, 0, 0,
        -1, 1, 0, 0,
        1, -1, 1, 1,
        1, 1, 1, 0
      ]),
      gl.STATIC_DRAW
    );

    const positionLocation = gl.getAttribLocation(program, "a_position");
    const texCoordLocation = gl.getAttribLocation(program, "a_texCoord");
    const stride = 4 * 4;
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, stride, 0);
    gl.enableVertexAttribArray(texCoordLocation);
    gl.vertexAttribPointer(texCoordLocation, 2, gl.FLOAT, false, stride, 2 * 4);

    this.bindWebGLTexture(gl, gl.TEXTURE0, program, "u_heightTexture", textures.height);
    this.bindWebGLTexture(gl, gl.TEXTURE1, program, "u_landTexture", textures.land);
    this.bindWebGLTexture(gl, gl.TEXTURE2, program, "u_fluxTexture", textures.flux);
    this.bindWebGLTexture(gl, gl.TEXTURE3, program, "u_overlayTexture", {
      width: overlay.width,
      height: overlay.height,
      texelSize: [1 / overlay.width, 1 / overlay.height],
      data: new Uint8Array(overlay.data)
    });

    gl.uniform2f(
      gl.getUniformLocation(program, "u_heightTexel"),
      textures.height.texelSize[0],
      textures.height.texelSize[1]
    );
    gl.uniform2f(
      gl.getUniformLocation(program, "u_landTexel"),
      textures.land.texelSize[0],
      textures.land.texelSize[1]
    );
    gl.uniform1f(gl.getUniformLocation(program, "u_hasFlux"), this.mapData.flux_map ? 1 : 0);

    gl.clearColor(0.04, 0.14, 0.25, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  private bindWebGLTexture(
    gl: WebGLRenderingContext | WebGL2RenderingContext,
    unit: number,
    program: WebGLProgram,
    uniformName: string,
    texture: EncodedRasterTexture
  ) {
    const glTexture = gl.createTexture();
    gl.activeTexture(unit);
    gl.bindTexture(gl.TEXTURE_2D, glTexture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      texture.width,
      texture.height,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      texture.data
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.uniform1i(gl.getUniformLocation(program, uniformName), unit - gl.TEXTURE0);
  }

  private async copyToWebGPU(imageData: ImageData) {
    const gpuNavigator = navigator as Navigator & { gpu?: GPU };
    if (!this.device || !this.context || !gpuNavigator.gpu) {
      throw new Error("WebGPU context is not ready");
    }

    const context = this.context as GPUCanvasContext;
    context.configure({
      device: this.device,
      format: gpuNavigator.gpu.getPreferredCanvasFormat(),
      alphaMode: "opaque"
    });

    const texture = this.device.createTexture({
      size: [imageData.width, imageData.height],
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT
    });

    this.device.queue.writeTexture(
      { texture },
      imageData.data,
      { bytesPerRow: imageData.width * 4 },
      { width: imageData.width, height: imageData.height }
    );

    const shaderModule = this.device.createShaderModule({
      code: `
        struct VertexOutput {
          @builtin(position) position: vec4f,
          @location(0) uv: vec2f,
        }
        @vertex
        fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
          var pos = array<vec2f, 6>(
            vec2f(-1.0, -1.0), vec2f(1.0, -1.0), vec2f(-1.0, 1.0),
            vec2f(-1.0, 1.0), vec2f(1.0, -1.0), vec2f(1.0, 1.0)
          );
          var uv = array<vec2f, 6>(
            vec2f(0.0, 1.0), vec2f(1.0, 1.0), vec2f(0.0, 0.0),
            vec2f(0.0, 0.0), vec2f(1.0, 1.0), vec2f(1.0, 0.0)
          );
          var output: VertexOutput;
          output.position = vec4f(pos[idx], 0.0, 1.0);
          output.uv = uv[idx];
          return output;
        }
        @group(0) @binding(0) var tex: texture_2d<f32>;
        @group(0) @binding(1) var samp: sampler;
        @fragment
        fn fs_main(input: VertexOutput) -> @location(0) vec4f {
          return textureSample(tex, samp, input.uv);
        }
      `
    });

    const pipeline = this.device.createRenderPipeline({
      layout: "auto",
      vertex: { module: shaderModule, entryPoint: "vs_main" },
      fragment: {
        module: shaderModule,
        entryPoint: "fs_main",
        targets: [{ format: gpuNavigator.gpu.getPreferredCanvasFormat() }]
      },
      primitive: {
        topology: "triangle-list"
      }
    });

    const sampler = this.device.createSampler({
      magFilter: "nearest",
      minFilter: "nearest",
      addressModeU: "clamp-to-edge",
      addressModeV: "clamp-to-edge"
    });

    const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: texture.createView() },
        { binding: 1, resource: sampler }
      ]
    });

    const commandEncoder = this.device.createCommandEncoder();
    const passEncoder = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: context.getCurrentTexture().createView(),
          loadOp: "clear",
          storeOp: "store",
          clearValue: { r: 1, g: 1, b: 1, a: 1 }
        }
      ]
    });

    passEncoder.setPipeline(pipeline);
    passEncoder.setBindGroup(0, bindGroup);
    passEncoder.draw(6);
    passEncoder.end();

    this.device.queue.submit([commandEncoder.finish()]);
  }

  private copyToWebGL(imageData: ImageData) {
    const gl = this.context as WebGLRenderingContext | WebGL2RenderingContext;
    gl.viewport(0, 0, imageData.width, imageData.height);

    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, imageData.width, imageData.height, 0, gl.RGBA, gl.UNSIGNED_BYTE, imageData.data);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    const vertexShader = this.createShader(
      gl,
      gl.VERTEX_SHADER,
      `
        attribute vec2 a_position;
        attribute vec2 a_texCoord;
        varying vec2 v_texCoord;
        void main() {
          gl_Position = vec4(a_position, 0.0, 1.0);
          v_texCoord = a_texCoord;
        }
      `
    );
    const fragmentShader = this.createShader(
      gl,
      gl.FRAGMENT_SHADER,
      `
        precision mediump float;
        varying vec2 v_texCoord;
        uniform sampler2D u_texture;
        void main() {
          gl_FragColor = texture2D(u_texture, v_texCoord);
        }
      `
    );

    const program = gl.createProgram();
    if (!program) {
      throw new Error("Failed to create WebGL program");
    }

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error(gl.getProgramInfoLog(program) ?? "Failed to link WebGL program");
    }

    gl.useProgram(program);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([
        -1, -1, 0, 1,
        1, -1, 1, 1,
        -1, 1, 0, 0,
        -1, 1, 0, 0,
        1, -1, 1, 1,
        1, 1, 1, 0
      ]),
      gl.STATIC_DRAW
    );

    const positionLocation = gl.getAttribLocation(program, "a_position");
    const texCoordLocation = gl.getAttribLocation(program, "a_texCoord");
    const stride = 4 * 4;
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, stride, 0);
    gl.enableVertexAttribArray(texCoordLocation);
    gl.vertexAttribPointer(texCoordLocation, 2, gl.FLOAT, false, stride, 2 * 4);

    const textureLocation = gl.getUniformLocation(program, "u_texture");
    gl.uniform1i(textureLocation, 0);
    gl.clearColor(1, 1, 1, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  private createShader(gl: WebGLRenderingContext | WebGL2RenderingContext, type: number, source: string) {
    const shader = gl.createShader(type);
    if (!shader) {
      throw new Error("Failed to allocate WebGL shader");
    }

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      const shaderType = type === gl.VERTEX_SHADER ? "vertex" : "fragment";
      const infoLog = gl.getShaderInfoLog(shader) ?? "Unknown shader compile error";
      gl.deleteShader(shader);
      throw new Error(`WebGL ${shaderType} shader compile failed: ${infoLog}`);
    }

    return shader;
  }

  private createProgram(
    gl: WebGLRenderingContext | WebGL2RenderingContext,
    vertexShader: WebGLShader,
    fragmentShader: WebGLShader
  ): WebGLProgram {
    const program = gl.createProgram();
    if (!program) {
      throw new Error("Failed to allocate WebGL program");
    }

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      const infoLog = gl.getProgramInfoLog(program) ?? "Unknown program link error";
      gl.deleteProgram(program);
      throw new Error(`WebGL program link failed: ${infoLog}`);
    }

    return program;
  }

  private async svgToPNG() {
    const svgString = await this.buildSVGString();
    const width = this.currentSvgWidth || this.mapData?.image_width || 0;
    const height = this.currentSvgHeight || this.mapData?.image_height || 0;
    if (!svgString || width <= 0 || height <= 0) {
      throw new Error("SVG has not been rendered yet");
    }

    const blob = new Blob([svgString], {
      type: "image/svg+xml;charset=utf-8"
    });
    const url = URL.createObjectURL(blob);

    return new Promise<string>((resolve, reject) => {
      const image = new Image();
      image.onload = () => {
        const canvas = document.createElement("canvas");
        canvas.width = width;
        canvas.height = height;
        const context = canvas.getContext("2d");
        if (!context) {
          URL.revokeObjectURL(url);
          reject(new Error("Failed to create export canvas"));
          return;
        }

        context.drawImage(image, 0, 0);
        URL.revokeObjectURL(url);
        resolve(canvas.toDataURL("image/png"));
      };
      image.onerror = () => {
        URL.revokeObjectURL(url);
        reject(new Error("Failed to rasterize SVG"));
      };
      image.src = url;
    });
  }
}
