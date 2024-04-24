import { PayloadAction, createSlice } from "@reduxjs/toolkit";

export interface Message {
  id: number;
  content: string;
  severity: "error" | "warning" | "info" | "success";
}

interface NotificationState {
  messages: Message[];
}

const initialState: NotificationState = {
  messages: [],
};

export const notificationReducer = createSlice({
  name: "notification",
  initialState,
  reducers: {
    showMessage: (state, action: PayloadAction<Message>) => {
      state.messages.push(action.payload);
    },
    hideMessage: (state, action: PayloadAction<number>) => {
      state.messages = state.messages.filter(
        (message) => message.id !== action.payload
      );
    },
  },
});

export const {showMessage, hideMessage} = notificationReducer.actions;
export default notificationReducer.reducer;