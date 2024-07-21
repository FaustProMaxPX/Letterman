import {
  Box,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Select,
  Typography,
} from "@mui/material";
import { GridColDef } from "@mui/x-data-grid";
import { Platform, Post } from "../../types";
import { NavIconButton } from "../common/NavIconButton";

import LinkIcon from "@mui/icons-material/Link";
import VisibilityIcon from "@mui/icons-material/Visibility";
import { useState } from "react";
import { useParams } from "react-router-dom";
import { PLATFORM_SET } from "../../constants";
import {
  forcePush,
  getSyncList,
  revertPost,
} from "../../services/postsService";
import { NotFoundDisplay } from "../common/NotFoundDisplay";
import { BasePage } from "../common/page/Page";
import { ConfirmDialog } from "../common/ConfirmDialog";
import useMessage from "../../hooks/useMessage";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { RouterBreadcurmbs } from "../RouterBreadcrumbs";
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
    width: 500,
    align: "center",
    valueGetter: (post: Post) =>
      post.content.length > 30
        ? post.content.slice(0, 30) + "..."
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
      <SyncOptionCell
        id={params.row.post.id}
        postId={params.row.post.postId}
        url={params.row.url}
        platform={params.row.platform}
      />
    ),
  },
];

interface SyncOptionCellProps {
  id: string;
  postId: string;
  url: string;
  platform: Platform;
}

const SyncOptionCell = ({ id, postId, url, platform }: SyncOptionCellProps) => {
  const [open, setOpen] = useState(false);
  const message = useMessage();
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
      <IconButton
        LinkComponent={"a"}
        href={url}
        target="_blank"
        rel="noreferrer"
        color="secondary"
      >
        <LinkIcon />
      </IconButton>
      <IconButton type="button" onClick={() => setOpen(true)}>
        <img width={20} height={20} alt="revert" src="/src/assets/revert.svg" />
      </IconButton>
      <ConfirmDialog
        open={open}
        onClose={() => setOpen(false)}
        onConfirm={() => {
          revertPost({ id })
            .then(() => {
              forcePush(postId, { platform })
                .then(() => {
                  message.success("文章已回滚成功");
                })
                .catch((err) =>
                  message.error(
                    `文章同步到${platform}失败：${formatErrorMessage(
                      err
                    )}，请手动重试`
                  )
                );
            })
            .catch((err) => message.error(formatErrorMessage(err)))
            .finally(() => setOpen(false));
        }}
        title="版本回滚"
        content={`是否确定回滚到该版本并推送到${platform}。\n
        注意：回滚文章将导致目标版本之后被创建的版本全部删除。`}
      />
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
      <RouterBreadcurmbs />
      <Box display={"flex"} justifyContent="space-between" mt={1} mb={2}>
        <Typography variant="h5">历史同步记录</Typography>
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
