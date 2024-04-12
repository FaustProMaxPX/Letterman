import { useDispatch } from "react-redux";
import { hideMessage, showMessage } from "../reducers/notificationReducer";
import { useCallback } from "react";

const useMessage = () => {
  const dispatch = useDispatch();
  const openMessage = useCallback(
    (content: string, duration = 3000) => {
      const id = Math.random();
      dispatch(showMessage({ id, content }));
      const timer = setTimeout(() => {
        dispatch(hideMessage(id));
      }, duration);
      return () => clearTimeout(timer);
    },
    [dispatch]
  );
  return openMessage;
};

export default useMessage;
