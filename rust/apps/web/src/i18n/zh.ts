import type { TranslationKeys } from './en';

export const zh: TranslationKeys = {
  title: '奇幻地图生成器',
  header: {
    generationTime: '生成时间',
    polygons: '多边形数',
    version: '版本',
    seed: '种子',
    ms: '毫秒',
  },
  config: {
    title: '地图配置',
    seed: '种子值',
    width: '宽度',
    height: '高度',
    resolution: '分辨率',
    cities: '城市数量',
    towns: '城镇数量',
    erosionSteps: '侵蚀步数',
    generate: '生成地图',
    generating: '生成中...',
    close: '关闭',
    openConfig: '配置',
  },
  layers: {
    title: '图层',
    contour: '等高线',
    rivers: '河流',
    slopes: '坡度',
    territory: '领土',
    cities: '城市',
    towns: '城镇',
    labels: '标签',
  },
  errors: {
    loadFailed: '地图数据加载失败',
    generateFailed: '地图生成失败',
    webgpuNotSupported: 'WebGPU 不支持，使用后备渲染器',
  },
  theme: {
    light: '浅色',
    dark: '深色',
  },
  lang: {
    en: 'English',
    zh: '中文',
  },
};
