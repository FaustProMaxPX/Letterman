import { IconButton, InputAdornment, TextField } from "@mui/material";
import { useContext } from "react";
import { FormContext } from "../../../hooks/useForm";

import DeleteIcon from "@mui/icons-material/Delete";
interface MapFieldProps {
  id: string;
  onDelete: (key: string) => void;
}

export const MapField = (props: MapFieldProps) => {
  const ctx = useContext(FormContext);
  const data: Map<string, string> = ctx?.values[props.id];
  return Array.from(data).map(([key, value]) => {
    let error = false;
    let msg = "";
    if (ctx?.errors[props.id] instanceof Map) {
      const map = ctx?.errors[props.id] as Map<string, string>;
      error = map.get(key) !== undefined;
      msg = map.get(key) || "";
    }
    return (
      <>
        <TextField
          key={key}
          id={key}
          label={key}
          onChange={(e) => {
            const value = e.target.value;
            const updated = new Map(ctx?.values[props.id]);
            updated.set(key, value);
            ctx?.setValue({ key: props.id, value: updated });
          }}
          variant="outlined"
          value={value}
          margin="normal"
          error={error}
          helperText={msg}
          InputProps={{
            endAdornment: (
              <InputAdornment position="end">
                <IconButton
                  aria-label="delete"
                  color="error"
                  onClick={() => props.onDelete(key)}
                >
                  <DeleteIcon />
                </IconButton>
              </InputAdornment>
            ),
          }}
        />
      </>
    );
  });
};
