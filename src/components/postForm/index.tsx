import { Button, Grid, Paper, TextField, Typography } from "@mui/material";
import { Post } from "../../types";
import { jsonToMap } from "../../utils/map-utils";
import React, { useState } from "react";
import { createPost } from "../../services/postsService";
import { useNavigate } from "react-router-dom";
import useMessage from "../../hooks/useMessage";
import CreateIcon from "@mui/icons-material/Create";
import { formatErrorMessage } from "../../services/utils/transform-response";
import ReactQuill from "react-quill";
import "react-quill/dist/quill.snow.css";

import TurndownService from "turndown";

export interface PostFormProps {
  post?: Post;
}

export const PostForm = ({ post }: PostFormProps) => {
  const navigate = useNavigate();
  const [title, setTitle] = useState(post === undefined ? "" : post.title);
  const [content, setContent] = useState(
    post === undefined ? "" : htmlToMarkdown(post.content)
  );

  const openMessage = useMessage();

  const handlePostSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createPost({
      title,
      content: htmlToMarkdown(content),
      metadata: JSON.parse("{}"),
    })
      .then(() => {
        navigate("/posts");
      })
      .catch((e: Error) => openMessage(formatErrorMessage(e)));
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
          sx={{ width: "100%", flexGrow: 1, height: "100%" }}
        >
          <Grid item xs={12} md={8} lg={9} mt={3}>
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
                  if (value === undefined) {
                    return false;
                  } else if (value.length === 0) {
                    return false;
                  } else {
                    return true;
                  }
                }}
                errorText="标题不能为空"
              />
              <ReactQuill
                theme="snow"
                value={content}
                onChange={(value) => {
                  setContent(value);
                }}
                style={{ marginTop: "10px", height: "80%" }}
              />
              
            </Paper>
          </Grid>
          <Grid item xs={12} md={4} lg={3} sx={{ mt: 3 }}>
            <Paper
              sx={{
                p: 3,
                display: "flex",
                flexDirection: "column",
                height: "100%",
              }}
            >
              <Typography variant="h6" color={"primary"} sx={{ mb: 2 }}>
                元数据
              </Typography>
              {Array.from(jsonToMap(post?.metadata)).map(([key, value]) => (
                <TextField
                  id={key}
                  label={key}
                  variant="outlined"
                  value={value}
                />
              ))}
              <TextField
                id="createTime"
                label="创建时间"
                variant="outlined"
                value={post?.createTime}
                contentEditable={"plaintext-only"}
                InputProps={{ readOnly: true }}
                margin="normal"
              />
              <TextField
                id="version"
                label="版本"
                variant="outlined"
                value={post?.version}
                InputProps={{ readOnly: true }}
                margin="normal"
              />
            </Paper>
          </Grid>
        </Grid>
        <Button
          variant="contained"
          startIcon={<CreateIcon />}
          type="submit"
          sx={{ mt: 3, ml: 1 }}
        >
          提交
        </Button>
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
  errorText?: string;
  validation?: (value: string | undefined) => boolean;
}

const FormField = (props: FormFieldProps) => {
  const [error, setError] = useState(false);
  const handleOnChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (props.validation !== undefined && !props.validation(e.target.value)) {
      setError(true);
    } else {
      setError(false);
    }
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
      helperText={error ? props.errorText : ""}
    />
  );
};

const htmlToMarkdown = (content: string) => {
  const turndownService = new TurndownService();
  return turndownService.turndown(content);
};
