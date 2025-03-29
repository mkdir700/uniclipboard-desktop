import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import { 
    getClipboardStats,
    ClipboardStats
} from '@/api/clipboardItems';

interface StatsState {
    stats: ClipboardStats;
    loading: boolean;
    error: string | null;
}

const initialState: StatsState = {
    stats: {
        total_items: 0,
        total_size: 0,
    },
    loading: false,
    error: null,
};

// 异步 Thunk Actions
export const fetchStats = createAsyncThunk<ClipboardStats, void>(
    'stats/fetchStats',
    async (_, { rejectWithValue }) => {
        try {
            const response = await getClipboardStats();
            return response;
        } catch (error) {
            return rejectWithValue(error instanceof Error ? error.message : '未知错误');
        }
    }
)

const statsSlice = createSlice({
    name: 'stats',
    initialState,
    reducers: {},
    extraReducers: (builder) => {
        builder.addCase(fetchStats.pending, (state) => {
            state.loading = true;
            state.error = null;
        })
        .addCase(fetchStats.fulfilled, (state, action) => {
            state.loading = false;
            state.stats = action.payload;
        })
        .addCase(fetchStats.rejected, (state, action) => {
            state.loading = false;
            state.error = action.error.message || '获取统计信息失败';
        });
    },
});

export const { } = statsSlice.actions;
export default statsSlice.reducer;