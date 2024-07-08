import { Editor } from "@bytemd/react";
import { useContext } from "react";
import { FormContext } from "../../../hooks/useForm";
import "bytemd/dist/index.css";
import "../../../css/bytemd.css";
import { BYTEMD_PLUGINS } from "../../../constants";

interface BytemdProps {
  id: string;
  sx?: React.CSSProperties;
}

export const BytemdField = (props: BytemdProps) => {
  const ctx = useContext(FormContext);
  return (
    <>
      <Editor
        value={ctx?.values[props.id] || ""}
        plugins={BYTEMD_PLUGINS}
        onChange={(value) => ctx?.setValue({ key: props.id, value: value })}
      />
    </>
  );
};
