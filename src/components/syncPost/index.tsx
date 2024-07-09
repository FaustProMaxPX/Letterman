import {
  Box,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  Typography,
} from "@mui/material";
import { GridColDef } from "@mui/x-data-grid";
import { Platform, Post } from "../../types";
import { NavIconButton } from "../common/NavIconButton";

import VisibilityIcon from "@mui/icons-material/Visibility";
import { useParams } from "react-router-dom";
import { getSyncList } from "../../services/postsService";
import { NotFoundDisplay } from "../common/NotFoundDisplay";
import { BasePage } from "../common/page/Page";
import { useState } from "react";
import { PLATFORM_SET } from "../../constants";
const columns: GridColDef[] = [
  {
    field: "version",
    headerName: "版本号",
    headerAlign: "center",
    minWidth: 100,
    align: "center",
  },
  {
    field: "post",
    headerName: "内容",
    headerAlign: "center",
    minWidth: 200,
    align: "center",
    valueGetter: (post: Post) =>
      post.content.length > 20
        ? post.content.slice(0, 20) + "..."
        : post.content,
  },
  {
    field: "createTime",
    headerName: "同步时间",
    headerAlign: "center",
    minWidth: 200,
    align: "center",
  },
  {
    field: "...",
    headerName: "...",
    headerAlign: "center",
    flex: 1,
    align: "center",
    renderCell: (params) => (
      <SyncOptionCell id={params.row.post.id} postId={params.row.post.postId} />
    ),
  },
];

const SyncOptionCell = ({ id, postId }: { id: string; postId: string }) => {
  return (
    <Box
      sx={{
        display: "flex",
        height: "100%",
        flexGrow: 1,
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <NavIconButton
        aria-label="see"
        color="primary"
        path={`/posts/sync/${postId}/detail/${id}`}
      >
        <VisibilityIcon />
      </NavIconButton>
    </Box>
  );
};

export const SyncPostPage = () => {
  const params = useParams();
  const id = params.id;
  const [platform, setPlatform] = useState<Platform>(Platform.Github);
  if (id === undefined) {
    return <NotFoundDisplay text="找不到文章" />;
  }
  return (
    <Box
      sx={{
        minHeight: 300,
        height: "100%",
        width: "100%",
        flexGrow: 1,
        overflow: "hidden",
      }}
    >
      <Box display={"flex"} justifyContent="space-between" mt={1} mb={2}>
        <Typography variant="h5">历史文章列表</Typography>
        <FormControl size="small">
          <InputLabel id="platform-inputlabel">Platform</InputLabel>
          <Select
            labelId="platform-selector-label"
            id="platform-selector"
            value={platform}
            onChange={(e) => setPlatform(e.target.value as Platform)}
            label="Platform"
          >
            {PLATFORM_SET.map((p) => (
              <MenuItem value={p}>{p}</MenuItem>
            ))}
          </Select>
        </FormControl>
      </Box>
      <BasePage
        colDef={columns}
        onPageChange={(page, pageSize) => {
          return getSyncList(id, { page, pageSize, platform });
        }}
      />
    </Box>
  );
};
