import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import { getInitialAppLanguage } from "@/lib/language";

const resources = {
  "zh-CN": {
    translation: {
      app: {
        title: "幻想地图生成器",
        subtitle: "WASM 驱动的交互式网页演示"
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
        layers: "图层控制"
      },
      actions: {
        generate: "生成地图",
        terrainOnly: "仅生成地形"
      },
      sections: {
        map: "地图",
        locations: "地点",
        rendering: "渲染",
        layers: "图层"
      },
      fields: {
        seed: "随机种子",
        size: "尺寸",
        detail: "细节级别",
        renderer: "渲染引擎",
        lineScale: "线条缩放",
        cities: "城市数量",
        towns: "城镇数量"
      },
      renderers: {
        auto: "自动",
        webgpu: "WebGPU",
        webgl: "WebGL",
        canvas: "Canvas 2D",
        svg: "SVG"
      },
      layers: {
        slope: "坡线",
        river: "河流",
        contour: "海岸线",
        border: "边境线",
        city: "城市",
        town: "城镇",
        label: "标签"
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
        exported: "导出完成"
      },
      stats: {
        cities: "城市",
        towns: "城镇",
        rivers: "河流",
        territories: "领地"
      },
      messages: {
        activeSeed: "实际种子：{{seed}}",
        imported: "地图数据已导入",
        exportedJson: "JSON 导出成功",
        exportedPng: "PNG 导出成功",
        exportedSvg: "SVG 导出成功",
        rendererReady: "当前渲染模式：{{mode}}"
      },
      errors: {
        wasmInit: "WASM 初始化失败：{{message}}",
        wasmUnavailable: "WASM 尚未就绪，请稍后重试",
        import: "加载文件失败：{{message}}",
        render: "地图渲染失败：{{message}}",
        export: "导出失败：{{message}}",
        generator: "地图生成失败：{{message}}"
      },
      helpers: {
        seedHint: "0 表示自动随机",
        detailHint: "建议范围 0.01 - 0.2",
        mobilePanel: "打开控制面板"
      }
    }
  },
  en: {
    translation: {
      app: {
        title: "Fantasy Map Generator",
        subtitle: "Interactive web demo powered by WASM"
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
        layers: "Layer control"
      },
      actions: {
        generate: "Generate map",
        terrainOnly: "Terrain only"
      },
      sections: {
        map: "Map",
        locations: "Locations",
        rendering: "Rendering",
        layers: "Layers"
      },
      fields: {
        seed: "Seed",
        size: "Size",
        detail: "Detail level",
        renderer: "Renderer",
        lineScale: "Line scale",
        cities: "Cities",
        towns: "Towns"
      },
      renderers: {
        auto: "Auto",
        webgpu: "WebGPU",
        webgl: "WebGL",
        canvas: "Canvas 2D",
        svg: "SVG"
      },
      layers: {
        slope: "Slope",
        river: "Rivers",
        contour: "Coastline",
        border: "Borders",
        city: "Cities",
        town: "Towns",
        label: "Labels"
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
        exported: "Export complete"
      },
      stats: {
        cities: "Cities",
        towns: "Towns",
        rivers: "Rivers",
        territories: "Territories"
      },
      messages: {
        activeSeed: "Active seed: {{seed}}",
        imported: "Map data imported",
        exportedJson: "JSON exported",
        exportedPng: "PNG exported",
        exportedSvg: "SVG exported",
        rendererReady: "Renderer mode: {{mode}}"
      },
      errors: {
        wasmInit: "Failed to initialize WASM: {{message}}",
        wasmUnavailable: "WASM is not ready yet. Please try again in a moment.",
        import: "Failed to load file: {{message}}",
        render: "Failed to render map: {{message}}",
        export: "Failed to export file: {{message}}",
        generator: "Failed to generate map: {{message}}"
      },
      helpers: {
        seedHint: "Use 0 for a random seed",
        detailHint: "Recommended range: 0.01 - 0.2",
        mobilePanel: "Open controls"
      }
    }
  }
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
    escapeValue: false
  }
});

export default i18n;
