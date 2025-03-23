import { invoke } from "@tauri-apps/api/core";

/**
 * 排序选项枚举
 */
export enum OrderBy {
  CreatedAtAsc = "created_at_asc",
  CreatedAtDesc = "created_at_desc",
  UpdatedAtAsc = "updated_at_asc",
  UpdatedAtDesc = "updated_at_desc",
  ContentTypeAsc = "content_type_asc",
  ContentTypeDesc = "content_type_desc",
  IsFavoritedAsc = "is_favorited_asc",
  IsFavoritedDesc = "is_favorited_desc"
}

export interface ClipboardItemResponse {
  id: string;
  device_id: string;
  content_type: string;
  display_content: string;
  is_downloaded: boolean;
  is_favorited: boolean;
  created_at: number;
  updated_at: number;
  content_size: number;
  is_truncated: boolean;
}

/**
 * 获取剪贴板历史记录
 * @param orderBy 排序方式
 * @param limit 限制返回的条目数
 * @param offset 偏移量，用于分页
 * @returns Promise，返回剪贴板条目数组
 */
export async function getClipboardItems(
  orderBy?: OrderBy,
  limit?: number, 
  offset?: number
): Promise<ClipboardItemResponse[]> {
  try {
    return await invoke('get_clipboard_items', { 
      order_by: orderBy, 
      limit, 
      offset 
    });
  } catch (error) {
    console.error('获取剪贴板历史记录失败:', error);
    throw error;
  }
}

/**
 * 获取单个剪贴板条目
 * @param id 剪贴板条目ID
 * @param fullContent 是否获取完整内容，不进行截断
 * @returns Promise，返回剪贴板条目，若不存在则返回null
 */
export async function getClipboardItem(id: string, fullContent: boolean = false): Promise<ClipboardItemResponse | null> {
  try {
    return await invoke('get_clipboard_item', { id, fullContent });
  } catch (error) {
    console.error('获取剪贴板条目失败:', error);
    throw error;
  }
}

/**
 * 删除剪贴板条目
 * @param id 剪贴板条目ID
 * @returns Promise，成功返回true
 */
export async function deleteClipboardItem(id: string): Promise<boolean> {
  try {
    return await invoke('delete_clipboard_item', { id });
  } catch (error) {
    console.error('删除剪贴板条目失败:', error);
    throw error;
  }
}

/**
 * 清空所有剪贴板历史记录
 * @returns Promise，成功返回删除的条目数
 */
export async function clearClipboardItems(): Promise<number> {
  try {
    return await invoke('clear_clipboard_items');
  } catch (error) {
    console.error('清空剪贴板历史记录失败:', error);
    throw error;
  }
}

/**
 * 复制剪贴板内容
 * @param id 剪贴板条目ID
 * @returns Promise，成功返回true
 */
export async function copyClipboardItem(id: string): Promise<boolean> {
  try {
    return await invoke('copy_clipboard_item', { id });
  } catch (error) {
    console.error('复制剪贴板记录失败:', error);
    throw error;
  }
}

/**
 * 根据内容类型获取符合前端显示的类型
 * @param contentType 内容类型字符串
 * @returns 适合UI显示的类型
 */
export function getDisplayType(contentType: string): "text" | "image" | "link" | "code" | "file" {
  if (contentType === "text") {
    return "text";
  } else if (contentType === "image") {
    return "image";
  } else if (contentType.includes("code") || contentType.includes("json")) {
    return "code";
  } else if (contentType.includes("link") || contentType.includes("url")) {
    return "link";
  } else {
    return "file";
  }
}

/**
 * 判断是否为图片类型
 * @param contentType 内容类型
 * @returns 是否为图片
 */
export function isImageType(contentType: string): boolean {
  return contentType === "image" || contentType.startsWith("image/");
}

/**
 * 判断是否为文本类型
 * @param contentType 内容类型
 * @returns 是否为文本
 */
export function isTextType(contentType: string): boolean {
  return contentType === "text" || contentType.startsWith("text/");
}
