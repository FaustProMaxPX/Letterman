import { Button, Grid, Paper, TextField, Typography } from "@mui/material";
import { Post } from "../../types";
import { jsonToMap } from "../../utils/map-utils";
import React, { useState } from "react";
import { createPost } from "../../services/postsService";
import { useNavigate } from "react-router-dom";
import useMessage from "../../hooks/useMessage";
import CreateIcon from "@mui/icons-material/Create";
import { formatErrorMessage } from "../../services/utils/transform-response";
export interface PostFormProps {
  post?: Post;
}

export const PostForm = ({ post }: PostFormProps) => {
  const navigate = useNavigate();
  const [title, setTitle] = useState(post === undefined ? "" : post.title);
  const [content, setContent] = useState(
    post === undefined ? "" : post.content
  );
  const openMessage = useMessage();

  const handlePostSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createPost({
      title,
      content,
      metadata: JSON.parse("{}"),
    })
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      .then((data) => {
        navigate("/posts");
      })
      .catch((e: Error) => openMessage(formatErrorMessage(e)));
  };
  return (
    <React.Fragment>
      <Typography component="h2" variant="h6" color="primary" gutterBottom>
        编辑文章
      </Typography>
      <form style={{ width: "100%", flexGrow: 1 }} onSubmit={handlePostSubmit}>
        <Grid container spacing={3} sx={{ width: "100%", flexGrow: 1 }}>
          <Grid item xs={12} md={8} lg={9} mt={3}>
            <Paper
              sx={{
                p: 3,
                display: "flex",
                flexDirection: "column",
              }}
              elevation={3}
            >
              <TextField
                id="title"
                label="标题"
                variant="outlined"
                fullWidth
                margin="normal"
                value={post?.title}
                onChange={(e) => setTitle(e.target.value)}
              />
              <TextField
                id="content"
                label="内容"
                variant="outlined"
                multiline
                rows={20}
                fullWidth
                margin="normal"
                value={post?.content}
                onChange={(e) => setContent(e.target.value)}
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
                ></TextField>
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
