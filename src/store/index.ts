import { configureStore } from '@reduxjs/toolkit';
import clipboardReducer from './slices/clipboardSlice';

export const store = configureStore({
  reducer: {
    clipboard: clipboardReducer,
    // 可以添加其他 reducer
  },
});

// 从 store 本身推断出 RootState 和 AppDispatch 类型
export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
