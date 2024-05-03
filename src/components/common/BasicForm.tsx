import { TextField } from "@mui/material";
import React, { createContext, useContext, useState } from "react";
import ReactQuill from "react-quill";

export interface ValidateResponse {
  success: boolean;
  message: string;
}

export interface BasicFormProps {
  initialValue: FormValues;
  children: React.ReactNode;
  onSubmit: (values: FormValues) => void;
  validate: (values: FormValues) => FormErrors;
  style: React.CSSProperties;
}

interface FormValues {
  [key: string]: any;
}

interface FormErrors {
  [key: string]: string | Map<string, string>;
}

interface UseFormProps {
  initialValue: FormValues;
  onSubmit: (values: FormValues) => void;
  validate: (values: FormValues) => FormErrors;
}

interface FormContextProps {
  values: FormValues;
  errors: FormErrors;
  handleChange: (e: { target: { id: string; value: any } }) => void;
  handleSubmit: (e: React.FormEvent) => void;
  setValue: ({ key, value }: { key: string; value: any }) => void;
}

export const FormContext = createContext<FormContextProps | null>(null);

export const useForm = ({ initialValue, onSubmit, validate }: UseFormProps) => {
  const [values, setValues] = useState(initialValue);
  const [errors, setErrors] = useState({});
  const handleChange = (e: { target: { id: string; value: any } }) => {
    const { id, value } = e.target;
    const newValues = { ...values, [id]: value };
    const newErrors = validate(newValues);
    setErrors(newErrors);
    setValues(newValues);
  };
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const newErrors = validate(values);
    if (Object.keys(newErrors).length === 0) {
      onSubmit(values);
    } else {
      setErrors(newErrors);
    }
  };
  const reset = () => {
    setValues(initialValue);
    setErrors({});
  };
  const setValue = ({ key, value }: { key: string; value: any }) => {
    setValues((prev) => ({ ...prev, [key]: value }));
  };
  return {
    values,
    errors,
    handleChange,
    handleSubmit,
    setValue,
    reset,
  };
};

export const BasicForm = (props: BasicFormProps) => {
  const formContext = useForm({
    initialValue: props.initialValue,
    onSubmit: props.onSubmit,
    validate: props.validate,
  });
  return (
    <FormContext.Provider value={formContext}>
      <form style={props.style} onSubmit={formContext.handleSubmit}>
        {props.children}
      </form>
    </FormContext.Provider>
  );
};

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

interface QuillProps {
  id: string;
  theme: string;
  style: React.CSSProperties;
}

export const QuillField = (props: QuillProps) => {
  const ctx = useContext(FormContext);
  return (
    <>
      <ReactQuill
        theme={props.theme}
        value={ctx?.values[props.id] || ""}
        onChange={(value) => ctx?.setValue({ key: props.id, value: value })}
        style={props.style}
      />
      {ctx?.errors[props.id] && (
        <div className="MuiFormHelperText-root Mui-error MuiFormHelperText-sizeMedium MuiFormHelperText-contained css-1wc848c-MuiFormHelperText-root">
          {ctx?.errors[props.id]}
        </div>
      )}
    </>
  );
};

interface MapFieldProps {
  id: string;
}

export const MapField = (props: MapFieldProps) => {
  const ctx = useContext(FormContext);
  const data: Map<string, string> = ctx?.values[props.id];
  return Array.from(data).map(([key, value]) => {
    let error = false;
    let msg = "";
    if (ctx?.errors[props.id] instanceof Map) {
      let map = ctx?.errors[props.id] as Map<string, string>;
      error = map.get(key) !== undefined;
      msg = map.get(key) || "";
    }
    return (
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
        fullWidth
        margin="normal"
        error={error}
        helperText={msg}
      />
    );
  });
};
