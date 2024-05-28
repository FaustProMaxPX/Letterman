import gfm from "@bytemd/plugin-gfm";
import { Editor } from "@bytemd/react";
import { useContext } from "react";
import { FormContext } from "../../../hooks/useForm";
import "bytemd/dist/index.css";
import "../../../css/bytemd.css"

interface BytemdProps {
  id: string;
  sx?: React.CSSProperties;
}

const plugins = [gfm()];

export const BytemdField = (props: BytemdProps) => {
  const ctx = useContext(FormContext);
  return (
    <>
      <Editor
        value={ctx?.values[props.id] || ""}
        plugins={plugins}
        onChange={(value) => ctx?.setValue({ key: props.id, value: value })}
      />
    </>
  );
};
