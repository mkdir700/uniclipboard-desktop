import { configureStore } from '@reduxjs/toolkit';
import clipboardReducer from './slices/clipboardSlice';
import statsReducer from './slices/statsSlice';

export const store = configureStore({
  reducer: {
    clipboard: clipboardReducer,
    stats: statsReducer,
  },
});

// 从 store 本身推断出 RootState 和 AppDispatch 类型
export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
