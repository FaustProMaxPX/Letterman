import { useDispatch } from "react-redux";
import { hideMessage, showMessage } from "../reducers/notificationReducer";
import { useCallback } from "react";

const useMessage = () => {
  const dispatch = useDispatch();
  const openMessage = useCallback(
    (
      content: string,
      duration = 2000,
      severity: "error" | "info" | "success" | "warning"
    ) => {
      const id = Math.random();
      dispatch(showMessage({ id, content, severity }));
      const timer = setTimeout(() => {
        dispatch(hideMessage(id));
      }, duration);
      return () => clearTimeout(timer);
    },
    [dispatch]
  );

  const error = useCallback(
    (content: string, duration?: number) =>
      openMessage(content, duration, "error"),
    [openMessage]
  );

  const info = useCallback(
    (content: string, duration?: number) =>
      openMessage(content, duration, "info"),
    [openMessage]
  );

  const warn = useCallback(
    (content: string, duration?: number) =>
      openMessage(content, duration, "warning"),
    [openMessage]
  );

  const success = useCallback(
    (content: string, duration?: number) =>
      openMessage(content, duration, "success"),
    [openMessage]
  );

  return { success, info, warn, error };
};

export default useMessage;
