import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  Grid,
  Paper,
  TextField,
  Typography,
} from "@mui/material";
import { Post } from "../../types";
import React, { useEffect, useRef, useState } from "react";
import { createPost, getPost } from "../../services/postsService";
import { useNavigate, useParams } from "react-router-dom";
import useMessage from "../../hooks/useMessage";
import CreateIcon from "@mui/icons-material/Create";
import { formatErrorMessage } from "../../services/utils/transform-response";
import ReactQuill from "react-quill";
import "react-quill/dist/quill.snow.css";

import TurndownService from "turndown";
import { formatDate } from "../../utils/time-util";
import { jsonToMap, mapToJson } from "../../utils/map-utils";
import { ValidateResponse } from "../types";

export interface PostFormProps {
  post?: Post;
}

export const PostForm = () => {
  const params = useParams();
  const id = params.id;
  const navigate = useNavigate();

  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [metadata, setMetadata] = useState<Map<string, string>>(new Map());
  const [open, setOpen] = useState(false);

  const postRef = useRef<Post>();
  const isUpdate = id !== undefined;

  useEffect(() => {
    if (id !== undefined) {
      getPost(id)
        .then((post_) => {
          setTitle(post_.title);
          setContent(post_.content);
          setMetadata(jsonToMap(post_.metadata));
          postRef.current = post_;
        })
        .catch((e: Error) => {
          message.error(formatErrorMessage(e));
        });
    }
  }, [id]);

  const message = useMessage();

  const handlePostSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createPost({
      title,
      content: htmlToMarkdown(content),
      metadata: mapToJson(metadata),
    })
      .then(() => {
        message.success("文章创建成功", 1000);
        setTimeout(() => navigate("/posts"), 1000);
      })
      .catch((e: Error) => message.error(formatErrorMessage(e)));
  };
  return (
    <React.Fragment>
      <Typography component="h2" variant="h6" color="primary" gutterBottom>
        编辑文章
      </Typography>
      <form
        style={{ width: "100%", height: "100%", flexGrow: 1 }}
        onSubmit={handlePostSubmit}
      >
        <Grid
          container
          spacing={3}
          sx={{ width: "100%", flexGrow: 1, height: "90%" }}
        >
          <Grid item xs={12} md={8} lg={9} >
            <Paper
              sx={{
                p: 3,
                display: "flex",
                flexDirection: "column",
                height: "100%",
              }}
              elevation={3}
            >
              <FormField
                id="title"
                label="标题"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                validation={(value) => {
                  if (value === undefined || value.length === 0) {
                    return { success: false, message: `标题不可为空` };
                  } else {
                    return { success: true, message: "" };
                  }
                }}
              />
              <ReactQuill
                theme="snow"
                value={content}
                onChange={(value) => {
                  setContent(value);
                }}
                style={{ marginTop: "10px", height: "40vh" }}
              />
            </Paper>
          </Grid>
          <Grid item xs={12} md={4} lg={3} >
            <Paper
              sx={{
                p: 3,
                display: "flex",
                flexDirection: "column",
                height: "100%",
                maxHeight: "72vh",
                overflow: "auto",
              }}
            >
              <Typography variant="h6" color={"primary"} >
                元数据
              </Typography>
              {Array.from(metadata).map(([key, value]) => {
                return (
                  <FormField
                    key={key}
                    id={key}
                    label={key}
                    value={value}
                    onChange={(e) => {
                      const updated = new Map(metadata);
                      updated.set(key, e.target.value);
                      setMetadata(updated);
                    }}
                    validation={(value) => {
                      if (value === undefined || value.length === 0) {
                        return { success: false, message: `${key} 不可为空` };
                      } else {
                        return { success: true, message: "" };
                      }
                    }}
                  />
                );
              })}
              {isUpdate && (
                <React.Fragment>
                  <Divider />
                  <TextField
                    id="createTime"
                    label="创建时间"
                    variant="outlined"
                    value={formatDate(postRef.current?.createTime)}
                    InputProps={{ readOnly: true }}
                    margin="normal"
                  />
                  <TextField
                    id="version"
                    label="版本"
                    variant="outlined"
                    value={postRef.current?.version || ""}
                    InputProps={{ readOnly: true }}
                    margin="normal"
                  />
                </React.Fragment>
              )}
            </Paper>
          </Grid>
        </Grid>
        <Button
          variant="contained"
          startIcon={<CreateIcon />}
          type="submit"
          sx={{ mt: 3 }}
        >
          提交
        </Button>
        <Button
          type="button"
          variant="contained"
          sx={{ float: "right", mt: 3, mr: 3 }}
          onClick={() => setOpen(true)}
        >
          添加新的元数据
        </Button>
        <MetadataDialogForm
          open={open}
          onClose={() => setOpen(false)}
          onSubmit={(key, value) =>
            setMetadata(new Map(metadata).set(key, value))
          }
          metadata={metadata}
        />
      </form>
    </React.Fragment>
  );
};

interface FormFieldProps {
  id: string;
  label: string;
  value?: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  rows?: number;
  multiline?: boolean;
  validation?: (value: string | undefined) => ValidateResponse;
}

const FormField = (props: FormFieldProps) => {
  const [error, setError] = useState(false);
  const [errorText, setErrorText] = useState("");

  const handleOnChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    let err = false;
    let msg = "";
    if (props.validation !== undefined) {
      const res = props.validation(e.target.value);
      if (!res.success) {
        err = true;
        msg = res.message;
      }
    }
    setError(err);
    setErrorText(msg);
    props.onChange(e);
  };

  return (
    <TextField
      id={props.id}
      label={props.label}
      variant="outlined"
      value={props.value}
      onChange={handleOnChange}
      fullWidth
      margin="normal"
      multiline={props.multiline}
      rows={props.rows}
      error={error}
      helperText={error ? errorText : ""}
    />
  );
};

interface MetadataDialogFormProps {
  open: boolean;
  onClose: () => void;
  onSubmit: (key: string, value: string) => void;
  metadata: Map<string, string>;
}

const MetadataDialogForm = (props: MetadataDialogFormProps) => {
  const [key, setKey] = useState("");
  const [value, setValue] = useState("");

  const handleAddMetadata = () => {
    props.onSubmit(key, value);
    setKey("");
    setValue("");
    props.onClose();
  };

  return (
    <Dialog
      open={props.open}
      onClose={props.onClose}
      PaperProps={{
        component: "form",
      }}
    >
      <DialogTitle>添加元数据</DialogTitle>
      <DialogContent>
        <FormField
          id="metadata-key"
          label="key"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          validation={(value) => {
            if (value === undefined || value.length === 0) {
              return { success: false, message: "key 不可为空" };
            } else if (props.metadata.has(value)) {
              return { success: false, message: `${value} 已存在` };
            } else {
              return { success: true, message: "" };
            }
          }}
        />
        <FormField
          id="metadata-value"
          label="value"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          validation={(value) => {
            if (value === undefined || value.length === 0) {
              return { success: false, message: "value 不可为空" };
            } else {
              return { success: true, message: "" };
            }
          }}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={handleAddMetadata}>添加</Button>
        <Button onClick={props.onClose}>取消</Button>
      </DialogActions>
    </Dialog>
  );
};

const htmlToMarkdown = (content: string) => {
  const turndownService = new TurndownService();
  return turndownService.turndown(content);
};
