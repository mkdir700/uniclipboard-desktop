import { invoke } from "@tauri-apps/api/core";


/**
 * 获取加密口令
 * @returns Promise，返回加密口令
 */
export async function getEncryptionPassword(): Promise<string> {
  try {
    return await invoke('get_encryption_password');
  } catch (error) {
    console.error('获取加密口令失败:', error);
    throw error;
  }
}

/**
 * 设置加密口令
 * @param password 要设置的加密口令
 * @returns Promise，成功返回true
 */
export async function setEncryptionPassword(password: string): Promise<boolean> {
  try {
    return await invoke('set_encryption_password', { password });
  } catch (error) {
    console.error('设置加密口令失败:', error);
    throw error;
  }
}

/**
 * 删除加密口令
 * @returns Promise，成功返回true
 */
export async function deleteEncryptionPassword(): Promise<boolean> {
  try {
    return await invoke('delete_encryption_password');
  } catch (error) {
    console.error('删除加密口令失败:', error);
    throw error;
  }
}