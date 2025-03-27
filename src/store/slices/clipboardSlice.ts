import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { 
  getClipboardItems, 
  deleteClipboardItem, 
  copyClipboardItem,
  clearClipboardItems,
  ClipboardItemResponse,
  OrderBy,
  favoriteClipboardItem,
  unfavoriteClipboardItem
} from '../../api/clipboardItems';

// 定义状态接口
interface ClipboardState {
  items: ClipboardItemResponse[];
  loading: boolean;
  error: string | null;
  deleteConfirmId: string | null;
}

// 初始状态
const initialState: ClipboardState = {
  items: [],
  loading: false,
  error: null,
  deleteConfirmId: null
};

// 定义获取剪贴板项目的参数接口
interface FetchClipboardItemsParams {
  orderBy?: OrderBy;
  limit?: number;
  offset?: number;
  isFavorited?: boolean;
}

// 异步 Thunk Actions
export const fetchClipboardItems = createAsyncThunk(
  'clipboard/fetchItems',
  async (params: FetchClipboardItemsParams = {}, { rejectWithValue }) => {
    try {
      return await getClipboardItems(
        params.orderBy,
        params.limit,
        params.offset,
        params.isFavorited
      );
    } catch (error) {
      return rejectWithValue("获取剪贴板内容失败");
    }
  }
);

export const removeClipboardItem = createAsyncThunk(
  'clipboard/removeItem',
  async (id: string, { rejectWithValue }) => {
    try {
      await deleteClipboardItem(id);
      return id;
    } catch (error) {
      return rejectWithValue('删除剪贴板内容失败');
    }
  }
);

// 目前API中没有toggleFavorite功能，先注释掉
export const toggleFavoriteItem = createAsyncThunk(
  'clipboard/toggleFavorite',
  async ({ id, isFavorited }: { id: string, isFavorited: boolean }, { rejectWithValue }) => {
    try {
      if (isFavorited) {
        await favoriteClipboardItem(id);
      } else {
        await unfavoriteClipboardItem(id);
      }
      return { id, isFavorited };
    } catch (error) {
      return rejectWithValue('设置收藏状态失败');
    }
  }
);

export const clearAllItems = createAsyncThunk(
  'clipboard/clearAll',
  async (_, { rejectWithValue }) => {
    try {
      await clearClipboardItems();
      return true;
    } catch (error) {
      return rejectWithValue('清空剪贴板内容失败');
    }
  }
);

export const copyToClipboard = createAsyncThunk(
  'clipboard/copyItem',
  async (id: string, { rejectWithValue }) => {
    try {
      const success = await copyClipboardItem(id);
      return { id, success };
    } catch (error) {
      return rejectWithValue('复制到剪贴板失败');
    }
  }
);

// 创建 Slice
const clipboardSlice = createSlice({
  name: 'clipboard',
  initialState,
  reducers: {
    setDeleteConfirmId: (state, action: PayloadAction<string | null>) => {
      state.deleteConfirmId = action.payload;
    },
    clearError: (state) => {
      state.error = null;
    }
  },
  extraReducers: (builder) => {
    // 处理获取剪贴板内容
    builder.addCase(fetchClipboardItems.pending, (state) => {
      state.loading = true;
      state.error = null;
    });
    builder.addCase(fetchClipboardItems.fulfilled, (state, action) => {
      state.loading = false;
      state.items = action.payload;
    });
    builder.addCase(fetchClipboardItems.rejected, (state, action) => {
      state.loading = false;
      state.error = action.payload as string;
    });

    // 处理删除剪贴板内容
    builder.addCase(removeClipboardItem.fulfilled, (state, action) => {
      state.items = state.items.filter(item => item.id !== action.payload);
      state.deleteConfirmId = null;
    });
    builder.addCase(removeClipboardItem.rejected, (state, action) => {
      state.error = action.payload as string;
    });

    // 处理收藏状态切换 - 暂时注释掉
    // builder.addCase(toggleFavoriteItem.fulfilled, (state, action) => {
    //   const { id, isFavorited } = action.payload;
    //   const item = state.items.find(item => item.id === id);
    //   if (item) {
    //     item.is_favorited = isFavorited;
    //   }
    // });
    // builder.addCase(toggleFavoriteItem.rejected, (state, action) => {
    //   state.error = action.payload as string;
    // });

    // 处理清空剪贴板
    builder.addCase(clearAllItems.fulfilled, (state) => {
      state.items = [];
    });
    builder.addCase(clearAllItems.rejected, (state, action) => {
      state.error = action.payload as string;
    });

    // 处理复制到剪贴板
    builder.addCase(copyToClipboard.rejected, (state, action) => {
      state.error = action.payload as string;
    });
  }
});

// 导出 Actions
export const { setDeleteConfirmId, clearError } = clipboardSlice.actions;

// 导出 Reducer
export default clipboardSlice.reducer;
