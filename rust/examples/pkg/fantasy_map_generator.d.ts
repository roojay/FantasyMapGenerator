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
    /**
     * 仅生成地形（不包括城市和边界）
     */
    generate_terrain_only(): string;
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

/**
 * 简化的地图生成函数（用于快速测试）
 */
export function generate_map_simple(seed: number, width: number, height: number): string;

/**
 * 设置 panic hook，在浏览器控制台显示 Rust panic 信息
 */
export function init_panic_hook(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmmapgenerator_free: (a: number, b: number) => void;
    readonly wasmmapgenerator_new: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmmapgenerator_generate: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmmapgenerator_generate_terrain_only: (a: number) => [number, number, number, number];
    readonly wasmmapgenerator_get_seed: (a: number) => number;
    readonly wasmmapgenerator_set_draw_scale: (a: number, b: number) => void;
    readonly generate_map_simple: (a: number, b: number, c: number) => [number, number, number, number];
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
