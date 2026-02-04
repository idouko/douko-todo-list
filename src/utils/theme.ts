/**
 * 将 hex 颜色转为 rgba 字符串（用于选中态等半透明背景）
 */
export function hexToRgba(hex: string, alpha: number): string {
  const h = hex.trim().replace(/^#/, "");
  if (h.length !== 6 && h.length !== 8) return hex;
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/**
 * 根据背景色亮度返回对比文字色：深底用白字，浅底用深字
 */
export function getContrastTextColor(backgroundColor: string): string {
  const hex = backgroundColor.trim().replace(/^#/, "");
  if (hex.length === 6 || hex.length === 8) {
    const r = parseInt(hex.slice(0, 2), 16) / 255;
    const g = parseInt(hex.slice(2, 4), 16) / 255;
    const b = parseInt(hex.slice(4, 6), 16) / 255;
    const luminance = 0.299 * r + 0.587 * g + 0.114 * b;
    return luminance < 0.4 ? "#ffffff" : "#1a1a1a";
  }
  const rgb = backgroundColor.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/);
  if (rgb) {
    const r = Number(rgb[1]) / 255;
    const g = Number(rgb[2]) / 255;
    const b = Number(rgb[3]) / 255;
    const luminance = 0.299 * r + 0.587 * g + 0.114 * b;
    return luminance < 0.4 ? "#ffffff" : "#1a1a1a";
  }
  return "#1a1a1a";
}
