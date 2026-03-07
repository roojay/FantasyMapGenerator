每次回答以【开发者】开头。
不要创建不必要的文档，文档应该保持简洁高效。

## 布局系统规范

### Grid 布局优先原则
优先使用 CSS Grid 而非 Flexbox，Grid 提供更强大的二维布局能力和更简洁的代码。

### 避免的 Flexbox 模式

❌ **不推荐**
```tsx
<Box className="flex h-full min-h-0 flex-col">
  <Box className="shrink-0">头部</Box>
  <Box className="flex-1 overflow-y-auto">内容</Box>
  <Box className="shrink-0">底部</Box>
</Box>
```

✅ **推荐**
```tsx
<Box className="grid h-full grid-rows-[auto_1fr_auto]">
  <Box>头部</Box>
  <Box className="overflow-y-auto">内容</Box>
  <Box>底部</Box>
</Box>
```

### 性能考虑
- Grid 在现代浏览器中性能优异
- 减少嵌套层级，Grid 可以直接定义复杂布局
- 避免使用 `min-h-0`、`min-w-0` 等 Flexbox hack

## UI 设计规范

### 工具栏统一样式
所有浮动工具栏（顶部、底部、左下角）使用一致的视觉风格：

```tsx
// 标准容器样式
className={cn(
  "rounded-lg border px-3 py-2 shadow-md",
  "backdrop-blur-xl"
)}
style={{
  backgroundColor: "var(--mantine-color-body)",
  borderColor: "rgb(var(--app-border))"
}}
```

**关键参数**
- 圆角: `rounded-lg` (8px)
- 内边距: `px-3 py-2` (12px 水平, 8px 垂直)
- 阴影: `shadow-md`
- 毛玻璃: `backdrop-blur-xl`
- 背景: `var(--mantine-color-body)`
- 边框: `rgb(var(--app-border))`

### 交互动画
所有可点击元素统一使用缩放动画：

```tsx
// 按钮/卡片
classNames={{
  root: cn(
    "cursor-pointer",
    "hover:scale-105 active:scale-95",
    "transition-transform duration-200"
  )
}}

// 图标按钮
classNames={{
  root: cn(
    "cursor-pointer",
    "hover:scale-110 active:scale-90",
    "transition-transform duration-200"
  )
}}
```

### 字体系统
使用 Inter UI 字体，具有出色的可读性：

```tsx
// 主字体
fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"

// 等宽数字
className="tabular-nums"
```

### 颜色系统
使用 CSS 变量实现主题切换：

```css
/* 浅色主题 */
--app-bg: 248 250 252;
--app-surface: 255 255 255;
--app-text: 15 23 42;
--app-muted: 100 116 139;
--app-border: 226 232 240;
--app-accent: 74 144 255;

/* 深色主题 */
--app-bg: 10 14 23;
--app-surface: 18 24 38;
--app-text: 241 245 249;
--app-muted: 148 163 184;
--app-border: 39 52 69;
```

### Hover 效果
复选框和列表项使用品牌色透明度：

```tsx
className={cn(
  "hover:bg-[rgba(var(--app-accent),0.08)]",
  "dark:hover:bg-[rgba(var(--app-accent),0.12)]",
  "transition-colors duration-200"
)}
```

### 分隔线
使用半透明边框实现分隔：

```tsx
<Box
  className="h-6 w-px"
  style={{ backgroundColor: "rgba(var(--app-border), 0.6)" }}
/>
```

### 组件布局
- 顶部工具栏: `top-4 right-4 lg:top-6 lg:right-6`
- 底部工具栏: `bottom-4 left-1/2 -translate-x-1/2 lg:bottom-6`
- 状态栏: `bottom-4 left-4 lg:bottom-6 lg:left-6`
- 统计信息: `top-4 left-1/2 -translate-x-1/2 lg:top-6`

### 响应式间距
- 移动端: `gap-xs` (4px), `p-sm` (12px)
- 桌面端: `gap-sm` (8px), `gap-md` (16px)

### 强制光标样式
所有交互元素必须显示小手光标：

```css
button,
[role="button"],
.mantine-Checkbox-root,
.mantine-ActionIcon-root,
.mantine-Button-root {
  cursor: pointer !important;
}
```
