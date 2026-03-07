# 示例输出目录

此目录用于存放地图生成器的输出文件。

## 文件类型

- `*.json` - 地图绘图数据（JSON 格式）
- `*.png` - 渲染后的地图图像

## 使用方法

### 生成示例地图

```bash
# 使用默认输出路径（examples/output）
cargo run --release --features render -- --seed 12345

# 自定义输出文件名
cargo run --release --features render -- --seed 12345 --output examples/my_map

# 不同的种子生成不同的地图
cargo run --release --features render -- --seed 54321 --output examples/map_54321
```

### 查看生成的文件

```bash
# 列出所有生成的文件
ls examples/

# 查看 JSON 数据
cat examples/output.json

# 打开 PNG 图像
# Windows: start examples/output.png
# Linux: xdg-open examples/output.png
# macOS: open examples/output.png
```

## 注意事项

- 此目录中的 `.json` 和 `.png` 文件会被 git 忽略
- 建议定期清理不需要的输出文件
- 输出文件名会自动添加扩展名（.json 和 .png）

## 清理输出

```bash
# 删除所有 JSON 文件
rm examples/*.json

# 删除所有 PNG 文件
rm examples/*.png

# 删除所有输出文件
rm examples/*.json examples/*.png
```
