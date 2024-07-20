import { Box, IconButton, Typography } from "@mui/material";
import { useParams } from "react-router-dom";
import { BasePage } from "../common/page/Page";
import { GridColDef } from "@mui/x-data-grid";
import { formatDate } from "../../utils/time-util";
import { NavIconButton } from "../common/NavIconButton";
import { ConfirmDialog } from "../common/ConfirmDialog";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { getPostHistory, revertPost } from "../../services/postsService";
import { useState } from "react";
import useMessage from "../../hooks/useMessage";
import VisibilityIcon from "@mui/icons-material/Visibility";
import { NotFoundDisplay } from "../common/NotFoundDisplay";

const columns: GridColDef[] = [
  {
    field: "title",
    headerName: "标题",
    headerAlign: "center",
    minWidth: 200,
    align: "center",
  },
  {
    field: "content",
    headerName: "内容",
    headerAlign: "center",
    minWidth: 400,
    align: "center",
    valueFormatter: (content: string) => {
      return content.length > 20 ? content.slice(0, 20) + "..." : content;
    },
  },
  {
    field: "version",
    headerName: "版本",
    headerAlign: "center",
    minWidth: 100,
    align: "center",
  },
  {
    field: "createTime",
    headerName: "创建时间",
    headerAlign: "center",
    minWidth: 200,
    align: "center",
    valueFormatter: (params: Date) => {
      return formatDate(params);
    },
  },
  {
    field: "...",
    headerName: "...",
    headerAlign: "center",
    flex: 1,
    align: "center",
    disableColumnMenu: true,
    renderCell: (params) => (
      <OptionCell id={params.row.id} postId={params.row.postId} />
    ),
  },
];

const OptionCell = ({ id, postId }: { id: string; postId: string }) => {
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
      <IconButton type="button" onClick={() => setOpen(true)}>
        <img
          width={20}
          height={20}
          alt="revert"
          src="/src/assets/revert.svg"
        />
      </IconButton>
      <ConfirmDialog
        open={open}
        onClose={() => setOpen(false)}
        onConfirm={() => {
          revertPost({ id })
            .then(() => {
              message.success("回滚成功");
            })
            .catch((err) => message.error(formatErrorMessage(err)))
            .finally(() => setOpen(false));
        }}
        title="版本回滚"
        content={`是否确定回滚到该版本。\n
        注意：回滚文章将导致目标版本之后被创建的版本全部删除。`}
      />
    </Box>
  );
};

export const PostHistoryPage = () => {
  const params = useParams();
  const postId = params.postId;
  if (postId === undefined) {
    return <NotFoundDisplay text="文章不存在" />;
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
      <Typography variant="h5" sx={{ mt: 1, mb: 2 }}>
        文章历史记录
      </Typography>
      <BasePage
        colDef={columns}
        onPageChange={(page, pageSize) => {
          return getPostHistory(postId, {
            page: page,
            pageSize: pageSize,
          });
        }}
      />
    </Box>
  );
};
