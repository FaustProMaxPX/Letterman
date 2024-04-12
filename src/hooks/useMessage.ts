import { useDispatch } from "react-redux";
import { hideMessage, showMessage } from "../reducers/notificationReducer";

const useMessage = () => {
  const dispatch = useDispatch();
  const openMessage = (content: string, duration = 3000) => {
    const id = Math.random();
    dispatch(showMessage({ id, content }));
    setTimeout(() => {
      dispatch(hideMessage(id));
    }, duration);
  };
  return openMessage;
};

export default useMessage;
