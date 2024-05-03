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
import { createPost, getPost, updatePost } from "../../services/postsService";
import { useNavigate, useParams } from "react-router-dom";
import useMessage from "../../hooks/useMessage";
import CreateIcon from "@mui/icons-material/Create";
import { formatErrorMessage } from "../../services/utils/transform-response";
import "react-quill/dist/quill.snow.css";

import { formatDate } from "../../utils/time-util";
import { jsonToMap, mapToJson } from "../../utils/map-utils";
import {
  FormContext,
  FormField,
  MapField,
  QuillField,
  useForm,
} from "../common/BasicForm";

export interface PostFormProps {
  post?: Post;
}

export const PostForm = () => {
  const params = useParams();
  const id = params.id;
  const navigate = useNavigate();

  const [open, setOpen] = useState(false);

  const postRef = useRef<Post>();
  const isUpdate = id !== undefined;
  const formCtx = useForm({
    initialValue: {
      title: "",
      content: "",
      metadata: new Map(),
    },
    validate: (values) => {
      const errors: {
        title?: string;
        content?: string;
        metadata?: Map<string, string>;
      } = {};
      if (values.title === undefined || values.title.length === 0) {
        errors.title = "文章标题不能为空";
      }
      if (values.content === undefined || values.content.length === 0) {
        errors.content = "文章内容不能为空";
      }
      if (values.metadata === undefined) {
        return errors;
      }
      const metadataErrors = new Map();
      for (const [key, value] of values.metadata.entries()) {
        if (value === undefined || value.length === 0) {
          metadataErrors.set(key, "value 不可为空");
        }
      }
      if (metadataErrors.size !== 0) {
        errors.metadata = metadataErrors;
      }
      return errors;
    },
    onSubmit: ({ title, content, metadata }) => {
      if (!isUpdate) {
        createPost({
          title,
          content,
          metadata: mapToJson(metadata),
        })
          .then(() => {
            message.success("文章创建成功", 1000);
            setTimeout(() => navigate("/posts"), 1000);
          })
          .catch((e: Error) => message.error(formatErrorMessage(e)));
      } else {
        updatePost({
          id,
          title,
          content,
          metadata: mapToJson(metadata),
        })
          .then(() => {
            message.success("文章更新成功", 1000);
            setTimeout(() => navigate("/posts"), 1000);
          })
          .catch((e: Error) => message.error(formatErrorMessage(e)));
      }
    },
  });

  useEffect(() => {
    if (id !== undefined) {
      getPost(id)
        .then((post_) => {
          formCtx.setValue({
            key: "title",
            value: post_.title,
          });
          formCtx.setValue({
            key: "content",
            value: post_.content,
          });
          formCtx.setValue({
            key: "metadata",
            value: jsonToMap(post_.metadata),
          });
          postRef.current = post_;
        })
        .catch((e: Error) => {
          message.error(formatErrorMessage(e));
        });
    }
  }, [id]);

  const message = useMessage();

  return (
    <React.Fragment>
      <Typography component="h2" variant="h6" color="primary" gutterBottom>
        编辑文章
      </Typography>
      <FormContext.Provider value={formCtx}>
        <form
          style={{ width: "100%", height: "100%", flexGrow: 1 }}
          onSubmit={formCtx.handleSubmit}
        >
          <Grid
            container
            spacing={3}
            sx={{ width: "100%", flexGrow: 1, height: "90%" }}
          >
            <Grid item xs={12} md={8} lg={9}>
              <Paper
                sx={{
                  p: 3,
                  display: "flex",
                  flexDirection: "column",
                  height: "100%",
                }}
                elevation={3}
              >
                <FormField id="title" label="标题" />
                <QuillField
                  id="content"
                  theme="snow"
                  style={{ marginTop: "10px", height: "40vh" }}
                />
              </Paper>
            </Grid>
            <Grid item xs={12} md={4} lg={3}>
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
                <Typography variant="h6" color={"primary"}>
                  元数据
                </Typography>
                <MapField id="metadata" />
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
        </form>
        <MetadataDialogForm
          open={open}
          onClose={() => setOpen(false)}
          onSubmit={(key, value) => {
            const updated = new Map(formCtx.values.metadata);
            updated.set(key, value);
            formCtx.setValue({ key: "metadata", value: updated });
          }}
          metadata={formCtx.values.metadata}
        />
      </FormContext.Provider>
    </React.Fragment>
  );
};

interface MetadataDialogFormProps {
  open: boolean;
  onClose: () => void;
  onSubmit: (key: string, value: string) => void;
  metadata: Map<string, string>;
}

const MetadataDialogForm = (props: MetadataDialogFormProps) => {
  const form = useForm({
    initialValue: {
      key: "",
      value: "",
    },
    validate: (values) => {
      const errors: { key?: string; value?: string } = {};
      if (values.key === undefined || values.key.length === 0) {
        errors.key = "key 不可为空";
      }
      if (props.metadata.has(values.key)) {
        errors.key = `${values.key}已存在`;
      }
      if (values.value === undefined || values.value.length === 0) {
        errors.value = "value 不可为空";
      }
      return errors;
    },
    onSubmit: (values) => {
      props.onSubmit(values.key, values.value);
      form.reset();
      props.onClose();
    },
  });

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
        <FormField id="key" label="key" />
        <FormField id="value" label="value" />
      </DialogContent>
      <DialogActions>
        <Button type="submit">添加</Button>
        <Button onClick={props.onClose}>取消</Button>
      </DialogActions>
    </Dialog>
  );
};
