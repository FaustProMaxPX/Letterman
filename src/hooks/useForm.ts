/* eslint-disable @typescript-eslint/no-explicit-any */
import { createContext, useState } from "react";

export interface ValidateResponse {
  success: boolean;
  message: string;
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