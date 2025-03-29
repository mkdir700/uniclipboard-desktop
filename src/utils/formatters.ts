/**
 * 格式化文件大小为人类可读格式
 * @param bytes 文件大小（字节）
 * @returns 格式化后的文件大小字符串
 */
export const formatFileSize = (bytes?: number): string => {
  if (bytes === undefined) return "未知大小";
  if (bytes === 0) return "0 字节";

  const units = ["字节", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
}; 