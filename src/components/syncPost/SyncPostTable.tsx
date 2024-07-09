import { useParams } from "react-router-dom";
import { Post } from "../../types";
import { useEffect, useState } from "react";
import { getPost } from "../../services/postsService";
import useMessage from "../../hooks/useMessage";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { Divider, Grid, Paper, TextField, Typography } from "@mui/material";
import { Box } from "@mui/system";
import { formatDate } from "../../utils/time-util";
import { Viewer } from "@bytemd/react";
import { BYTEMD_PLUGINS } from "../../constants";
import "bytemd/dist/index.css";
import { jsonToMap } from "../../utils/map-utils";

export const SyncPostTable = () => {
  const params = useParams();
  const id = params.id;
  const [post, setPost] = useState<Post | undefined>(undefined);
  const message = useMessage();
  useEffect(() => {
    if (id === undefined) return;
    getPost(id)
      .then((data) => setPost(data))
      .catch((err) => {
        message.error(formatErrorMessage(err));
      });
  }, [id]);

  return (
    <>
      <Typography component="h2" variant="h6" color="primary" gutterBottom>
        查看历史文章
      </Typography>
      <Box sx={{ width: "100%", height: "100%", flexGrow: 1 }}>
        <Grid
          container
          spacing={3}
          sx={{ width: "100%", height: "90%", flexGrow: 1 }}
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
              <TextField
                id="title"
                label="标题"
                value={post?.title || ""}
                InputProps={{ readOnly: true }}
              />
              <Viewer value={post?.content || ""} plugins={BYTEMD_PLUGINS} />
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
              {Array.from(jsonToMap(post?.metadata)).map(([key, value]) => (
                <>
                  <TextField
                    key={key}
                    id={key}
                    label={key}
                    variant="outlined"
                    value={value}
                    margin="normal"
                    InputProps={{ readOnly: true }}
                  />
                </>
              ))}
              <Divider />
              <TextField
                id="createTime"
                label="创建时间"
                variant="outlined"
                value={formatDate(post?.createTime)}
                InputProps={{ readOnly: true }}
                margin="normal"
              />
              <TextField
                id="version"
                label="版本"
                variant="outlined"
                value={post?.version || ""}
                InputProps={{ readOnly: true }}
                margin="normal"
              />
            </Paper>
          </Grid>
        </Grid>
      </Box>
    </>
  );
};
