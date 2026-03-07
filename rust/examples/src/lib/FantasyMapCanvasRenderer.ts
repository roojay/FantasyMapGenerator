import { generateSVGString } from "@/lib/svgBuilder";
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
  private svgElement: SVGSVGElement | null = null;
  private renderMode: RenderBackend | null = null;
  private availableModes: RenderBackend[] = [];
  private mapData: MapData | null = null;
  private layers: MapLayers = { ...DEFAULT_LAYERS };

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

    if (this.svgElement) {
      this.svgElement.remove();
      this.svgElement = null;
    }

    this.context = null;
    this.renderMode = null;
  }

  loadMapData(mapData: string | MapData) {
    try {
      this.mapData = typeof mapData === "string" ? (JSON.parse(mapData) as MapData) : mapData;
      this.resetDrawScale();
      return true;
    } catch {
      return false;
    }
  }

  setLayers(layers: MapLayers) {
    this.layers = { ...layers };
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
      this.renderToSVG(width, height);
      return;
    }

    this.canvas.style.display = "block";
    this.canvas.width = width;
    this.canvas.height = height;
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;

    if (this.renderMode === "canvas") {
      this.renderToCanvas2D(this.context as CanvasRenderingContext2D, width, height);
      this.canvas.style.visibility = "visible";
      return;
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

  buildSVGString() {
    if (!this.mapData) {
      throw new Error("No map data to export");
    }

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
    context.fillStyle = this.rgbaToStyle(this.BACKGROUND_RGBA);
    context.fillRect(0, 0, width, height);

    if (this.layers.slope) {
      context.lineWidth = this.SLOPE_LINE_WIDTH;
      context.strokeStyle = this.rgbaToStyle(this.SLOPE_RGBA);
      this.drawSegments(this.mapData?.slope ?? [], context, width, height);
    }

    if (this.layers.border) {
      context.lineWidth = this.BORDER_LINE_WIDTH;
      context.strokeStyle = this.rgbaToStyle(this.BACKGROUND_RGBA);
      this.drawPaths(this.mapData?.territory ?? [], context, width, height);

      context.lineWidth = this.BORDER_LINE_WIDTH;
      context.setLineDash(this.BORDER_DASH_PATTERN);
      context.lineCap = "butt";
      context.lineJoin = "bevel";
      context.strokeStyle = this.rgbaToStyle(this.BORDER_RGBA);
      this.drawPaths(this.mapData?.territory ?? [], context, width, height);
      context.setLineDash([]);
    }

    if (this.layers.river) {
      context.lineWidth = this.RIVER_LINE_WIDTH;
      context.strokeStyle = this.rgbaToStyle(this.RIVER_RGBA);
      this.drawPaths(this.mapData?.river ?? [], context, width, height);
    }

    if (this.layers.contour) {
      context.lineWidth = this.CONTOUR_LINE_WIDTH;
      context.strokeStyle = this.rgbaToStyle(this.CONTOUR_RGBA);
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
      context.fillStyle = this.rgbaToStyle(this.CITY_MARKER_RGBA);
      context.beginPath();
      context.arc(x, y, this.CITY_MARKER_OUTER_RADIUS, 0, Math.PI * 2);
      context.fill();

      context.fillStyle = this.rgbaToStyle(this.BACKGROUND_RGBA);
      context.beginPath();
      context.arc(x, y, this.CITY_MARKER_INNER_RADIUS, 0, Math.PI * 2);
      context.fill();
    }
  }

  private drawTowns(data: number[], context: CanvasRenderingContext2D, width: number, height: number) {
    context.fillStyle = this.rgbaToStyle(this.TOWN_MARKER_RGBA);
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
      const x = label.position[0] * width;
      const y = (1 - label.position[1]) * height;
      const minx = label.extents[0] * width;
      const miny = (1 - label.extents[1]) * height;
      const maxx = label.extents[2] * width;
      const maxy = (1 - label.extents[3]) * height;

      context.fillStyle = this.rgbaToStyle(this.BACKGROUND_RGBA);
      context.fillRect(minx, maxy, Math.abs(maxx - minx), Math.abs(maxy - miny));
      context.fillStyle = this.rgbaToStyle(this.TEXT_RGBA);
      context.fillText(label.text, x, y);
    }
  }

  private renderToSVG(width: number, height: number) {
    this.canvas.style.display = "none";
    this.canvas.width = width;
    this.canvas.height = height;

    this.svgElement?.remove();
    const svgString = generateSVGString(this.mapData as MapData, this.getSvgStyles(), this.layers);
    const parser = new DOMParser();
    const parsed = parser.parseFromString(svgString, "image/svg+xml");
    this.svgElement = parsed.documentElement as unknown as SVGSVGElement;
    this.svgElement.classList.add("block", "bg-white");
    this.stage.appendChild(this.svgElement);
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

    if (!vertexShader || !fragmentShader) {
      throw new Error("Failed to compile WebGL shaders");
    }

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
      return null;
    }

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  }

  private svgToPNG() {
    if (!this.svgElement) {
      throw new Error("SVG has not been rendered yet");
    }

    const blob = new Blob([new XMLSerializer().serializeToString(this.svgElement)], {
      type: "image/svg+xml;charset=utf-8"
    });
    const url = URL.createObjectURL(blob);

    return new Promise<string>((resolve, reject) => {
      const image = new Image();
      image.onload = () => {
        const canvas = document.createElement("canvas");
        canvas.width = Number(this.svgElement?.getAttribute("width") ?? 0);
        canvas.height = Number(this.svgElement?.getAttribute("height") ?? 0);
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
