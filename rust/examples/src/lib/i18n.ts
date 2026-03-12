import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import { getInitialAppLanguage } from "@/lib/language";

const resources = {
  "zh-CN": {
    translation: {
      app: {
        title: "幻想地图生成器",
        subtitle: "WASM 驱动的交互式网页演示",
      },
      toolbar: {
        import: "导入",
        export: "导出",
        language: "语言",
        theme: "主题",
        light: "浅色",
        dark: "深色",
        fit: "适应视图",
        reset: "重置视图",
        hideSidebar: "隐藏侧栏",
        showSidebar: "显示侧栏",
        layers: "图层控制",
      },
      actions: {
        generate: "生成地图",
        terrainOnly: "仅生成地形",
        importPreset: "导入预设",
        exportPreset: "导出预设",
        resetPreset: "重置预设",
        toggle: "切换",
      },
      common: {
        on: "开启",
        off: "关闭",
      },
      sections: {
        map: "地图",
        locations: "地点",
        rendering: "渲染",
        layers: "图层",
      },
      fields: {
        seed: "随机种子",
        size: "尺寸",
        detail: "细节级别",
        renderer: "渲染引擎",
        lineScale: "线条缩放",
        cities: "城市数量",
        towns: "城镇数量",
      },
      renderers: {
        auto: "自动",
        webgpu: "WebGPU",
        svg: "SVG",
      },
      layers: {
        slope: "坡线",
        river: "河流",
        contour: "海岸线",
        border: "边境线",
        city: "城市",
        town: "城镇",
        label: "标签",
      },
      status: {
        booting: "正在初始化 WASM…",
        wasmReady: "WASM 已就绪",
        ready: "准备就绪",
        generating: "正在生成地图…",
        rendering: "正在渲染地图…",
        rendered: "地图已渲染",
        importing: "正在加载地图数据…",
        switching: "正在切换渲染器…",
        exportingJSON: "正在导出 JSON…",
        exportingPNG: "正在导出 PNG…",
        exportingSVG: "正在导出 SVG…",
        exportingPreset: "正在导出展示预设…",
        exported: "导出完成",
      },
      stats: {
        cities: "城市",
        towns: "城镇",
        rivers: "河流",
        territories: "领地",
      },
      messages: {
        activeSeed: "实际种子：{{seed}}",
        imported: "地图数据已导入",
        exportedJson: "JSON 导出成功",
        exportedPng: "PNG 导出成功",
        exportedSvg: "SVG 导出成功",
        presetImported: "展示预设已导入",
        presetExported: "展示预设已导出",
        presetReset: "展示预设已重置",
        rendererReady: "当前渲染模式：{{mode}}",
        switchingHint: "保留当前画面，正在切换至 {{mode}}",
        generatingHint: "正在后台生成地形、河流、城镇与标签数据，请稍候。",
        importingHint: "正在读取并解析地图文件内容，请稍候。",
        renderingHint: "正在整理场景资源并绘制首帧预览。",
        exportingHint: "正在准备 {{format}} 文件并写出结果，请勿操作界面。",
        presetExportHint: "正在写出展示预设文件，请勿操作界面。",
        operationLocked: "正在处理当前地图任务，界面暂时锁定以避免误操作。",
        operationLockedShort: "Processing",
      },
      errors: {
        wasmInit: "WASM 初始化失败：{{message}}",
        wasmUnavailable: "WASM 尚未就绪，请稍后重试",
        import: "加载文件失败：{{message}}",
        presetImport: "加载展示预设失败：{{message}}",
        render: "地图渲染失败：{{message}}",
        export: "导出失败：{{message}}",
        generator: "地图生成失败：{{message}}",
      },
      helpers: {
        seedHint: "0 表示自动随机",
        detailHint: "建议范围 0.01 - 0.2",
        mobilePanel: "打开控制面板",
      },
    },
  },
  en: {
    translation: {
      app: {
        title: "Fantasy Map Generator",
        subtitle: "Interactive web demo powered by WASM",
      },
      toolbar: {
        import: "Import",
        export: "Export",
        language: "Language",
        theme: "Theme",
        light: "Light",
        dark: "Dark",
        fit: "Fit view",
        reset: "Reset view",
        hideSidebar: "Hide sidebar",
        showSidebar: "Show sidebar",
        layers: "Layer control",
      },
      actions: {
        generate: "Generate map",
        terrainOnly: "Terrain only",
        importPreset: "Import preset",
        exportPreset: "Export preset",
        resetPreset: "Reset preset",
        toggle: "Toggle",
      },
      common: {
        on: "On",
        off: "Off",
      },
      sections: {
        map: "Map",
        locations: "Locations",
        rendering: "Rendering",
        layers: "Layers",
      },
      fields: {
        seed: "Seed",
        size: "Size",
        detail: "Detail level",
        renderer: "Renderer",
        lineScale: "Line scale",
        cities: "Cities",
        towns: "Towns",
      },
      renderers: {
        auto: "Auto",
        webgpu: "WebGPU",
        svg: "SVG",
      },
      layers: {
        slope: "Slope",
        river: "Rivers",
        contour: "Coastline",
        border: "Borders",
        city: "Cities",
        town: "Towns",
        label: "Labels",
      },
      status: {
        booting: "Initializing WASM…",
        wasmReady: "WASM ready",
        ready: "Ready",
        generating: "Generating map…",
        rendering: "Rendering map…",
        rendered: "Map rendered",
        importing: "Loading map data…",
        switching: "Switching renderer…",
        exportingJSON: "Exporting JSON…",
        exportingPNG: "Exporting PNG…",
        exportingSVG: "Exporting SVG…",
        exportingPreset: "Exporting preset…",
        exported: "Export complete",
      },
      stats: {
        cities: "Cities",
        towns: "Towns",
        rivers: "Rivers",
        territories: "Territories",
      },
      messages: {
        activeSeed: "Active seed: {{seed}}",
        imported: "Map data imported",
        exportedJson: "JSON exported",
        exportedPng: "PNG exported",
        exportedSvg: "SVG exported",
        presetImported: "Presentation preset imported",
        presetExported: "Presentation preset exported",
        presetReset: "Presentation preset reset",
        rendererReady: "Renderer mode: {{mode}}",
        switchingHint: "Keeping the current frame visible while switching to {{mode}}",
        generatingHint: "Generating terrain, rivers, settlements, and labels in the background.",
        importingHint: "Reading and parsing the map file content.",
        renderingHint: "Preparing scene resources and drawing the first preview frame.",
        exportingHint:
          "Preparing the {{format}} file and writing the result. Please avoid interacting with the UI.",
        presetExportHint:
          "Writing the presentation preset file. Please avoid interacting with the UI.",
        operationLocked:
          "The interface is temporarily locked while the map task finishes to avoid accidental input.",
        operationLockedShort: "Processing",
      },
      errors: {
        wasmInit: "Failed to initialize WASM: {{message}}",
        wasmUnavailable: "WASM is not ready yet. Please try again in a moment.",
        import: "Failed to load file: {{message}}",
        presetImport: "Failed to load presentation preset: {{message}}",
        render: "Failed to render map: {{message}}",
        export: "Failed to export file: {{message}}",
        generator: "Failed to generate map: {{message}}",
      },
      helpers: {
        seedHint: "Use 0 for a random seed",
        detailHint: "Recommended range: 0.01 - 0.2",
        mobilePanel: "Open controls",
      },
    },
  },
};

const initialLanguage = getInitialAppLanguage();

if (typeof document !== "undefined") {
  document.documentElement.lang = initialLanguage;
}

void i18n.use(initReactI18next).init({
  resources,
  lng: initialLanguage,
  fallbackLng: "en",
  initImmediate: false,
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
