import { TextField } from "@mui/material";
import { useContext } from "react";
import { FormContext } from "../../../hooks/useForm";

interface FormFieldProps {
  id: string;
  label: string;
  rows?: number;
  multiline?: boolean;
}

export const FormField = (props: FormFieldProps) => {
  const ctx = useContext(FormContext);
  return (
    <TextField
      id={props.id}
      label={props.label}
      variant="outlined"
      value={ctx?.values[props.id] || ""}
      onChange={ctx?.handleChange}
      fullWidth
      margin="normal"
      multiline={props.multiline}
      rows={props.rows}
      error={ctx?.errors[props.id] !== undefined}
      helperText={
        ctx?.errors[props.id] !== undefined ? ctx?.errors[props.id] : ""
      }
    />
  );
};
