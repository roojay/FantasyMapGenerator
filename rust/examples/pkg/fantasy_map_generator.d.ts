/* tslint:disable */
/* eslint-disable */

/**
 * WASM 地图生成器包装器
 */
export class WasmMapGenerator {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 生成完整的地图
     *
     * 返回包含地图数据的 JSON 字符串
     */
    generate(num_cities: number, num_towns: number): string;
    generate_render_packet(num_cities: number, num_towns: number): WasmRenderPacket;
    /**
     * 仅生成地形（不包括城市和边界）
     */
    generate_terrain_only(): string;
    /**
     * 生成完整地图，并允许网页端按需决定是否导出附加栅格数据。
     *
     * # 参数
     * * `num_cities` - 城市数量
     * * `num_towns` - 城镇数量
     * * `include_raster_data` - 是否导出附加栅格数据
     *
     * # 与原始 C++ 的差异
     * 原始 C++ 版本没有 WASM 导出层，也没有“按需导出栅格”的调用入口。
     * 这个接口是本 fork 为浏览器场景新增的能力，用来减少 WASM 与 JS 之间
     * 的大块 JSON 传输。
     *
     * # 性能说明
     * 普通矢量渲染可将 `include_raster_data` 设为 `false`，
     * 仅在需要额外栅格数据时再开启。
     */
    generate_with_options(num_cities: number, num_towns: number, include_raster_data: boolean): string;
    /**
     * 获取当前使用的种子
     */
    get_seed(): number;
    /**
     * 创建新的地图生成器
     *
     * # 参数
     * - `seed`: 随机种子（0 表示使用时间戳）
     * - `width`: 地图宽度（像素）
     * - `height`: 地图高度（像素）
     * - `resolution`: 网格分辨率（0.01-0.2，推荐 0.08）
     */
    constructor(seed: number, width: number, height: number, resolution: number);
    /**
     * 设置绘制缩放比例
     */
    set_draw_scale(scale: number): void;
}

export class WasmRenderPacket {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    ao_texture(): Uint8Array;
    border_offsets(): Uint32Array;
    border_positions(): Float32Array;
    city_positions(): Float32Array;
    coast_glow_texture(): Uint8Array;
    contour_offsets(): Uint32Array;
    contour_positions(): Float32Array;
    flux_texture(): Uint8Array;
    height_texture(): Uint8Array;
    label_anchors(): Float32Array;
    label_bytes(): Uint8Array;
    label_offsets(): Uint32Array;
    label_sizes(): Float32Array;
    land_mask_texture(): Uint8Array;
    land_polygon_offsets(): Uint32Array;
    land_polygon_positions(): Float32Array;
    river_offsets(): Uint32Array;
    river_positions(): Float32Array;
    roughness_texture(): Uint8Array;
    slope_segments(): Float32Array;
    terrain_albedo_texture(): Uint8Array;
    terrain_indices(): Uint32Array;
    terrain_normals(): Float32Array;
    terrain_positions(): Float32Array;
    terrain_uvs(): Float32Array;
    town_positions(): Float32Array;
    water_alpha_texture(): Uint8Array;
    water_color_texture(): Uint8Array;
    readonly metadata_json: string;
    readonly svg_json: string;
}

/**
 * 根据导出的地图 JSON 和图层配置在 Rust 侧生成标准原始地图 SVG。
 */
export function build_map_svg(map_json: string, layers_json: string): string;

/**
 * 简化的地图生成函数（用于快速测试）
 */
export function generate_map_simple(seed: number, width: number, height: number): string;

/**
 * 设置 panic hook，在浏览器控制台显示 Rust panic 信息
 */
export function init_panic_hook(): void;

/**
 * 返回 presentation 插件的统一 capability/config metadata。
 */
export function presentation_plugin_metadata_json(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmmapgenerator_free: (a: number, b: number) => void;
    readonly __wbg_wasmrenderpacket_free: (a: number, b: number) => void;
    readonly build_map_svg: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly generate_map_simple: (a: number, b: number, c: number) => [number, number, number, number];
    readonly presentation_plugin_metadata_json: () => [number, number, number, number];
    readonly wasmmapgenerator_generate: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmmapgenerator_generate_render_packet: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmmapgenerator_generate_terrain_only: (a: number) => [number, number, number, number];
    readonly wasmmapgenerator_generate_with_options: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly wasmmapgenerator_get_seed: (a: number) => number;
    readonly wasmmapgenerator_new: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmmapgenerator_set_draw_scale: (a: number, b: number) => void;
    readonly wasmrenderpacket_ao_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_border_offsets: (a: number) => [number, number];
    readonly wasmrenderpacket_border_positions: (a: number) => [number, number];
    readonly wasmrenderpacket_city_positions: (a: number) => [number, number];
    readonly wasmrenderpacket_coast_glow_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_contour_offsets: (a: number) => [number, number];
    readonly wasmrenderpacket_contour_positions: (a: number) => [number, number];
    readonly wasmrenderpacket_flux_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_height_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_label_anchors: (a: number) => [number, number];
    readonly wasmrenderpacket_label_bytes: (a: number) => [number, number];
    readonly wasmrenderpacket_label_offsets: (a: number) => [number, number];
    readonly wasmrenderpacket_label_sizes: (a: number) => [number, number];
    readonly wasmrenderpacket_land_mask_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_land_polygon_offsets: (a: number) => [number, number];
    readonly wasmrenderpacket_land_polygon_positions: (a: number) => [number, number];
    readonly wasmrenderpacket_metadata_json: (a: number) => [number, number];
    readonly wasmrenderpacket_river_offsets: (a: number) => [number, number];
    readonly wasmrenderpacket_river_positions: (a: number) => [number, number];
    readonly wasmrenderpacket_roughness_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_slope_segments: (a: number) => [number, number];
    readonly wasmrenderpacket_svg_json: (a: number) => [number, number];
    readonly wasmrenderpacket_terrain_albedo_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_terrain_indices: (a: number) => [number, number];
    readonly wasmrenderpacket_terrain_normals: (a: number) => [number, number];
    readonly wasmrenderpacket_terrain_positions: (a: number) => [number, number];
    readonly wasmrenderpacket_terrain_uvs: (a: number) => [number, number];
    readonly wasmrenderpacket_town_positions: (a: number) => [number, number];
    readonly wasmrenderpacket_water_alpha_texture: (a: number) => [number, number];
    readonly wasmrenderpacket_water_color_texture: (a: number) => [number, number];
    readonly init_panic_hook: () => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
