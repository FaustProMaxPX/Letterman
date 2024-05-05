import {
  Box,
  Button,
  IconButton,
  IconButtonProps,
  Typography,
} from "@mui/material";
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { useEffect, useState } from "react";
import { DEFAULT_PAGE, DEFAULT_PAGE_SIZE } from "../../constants";
import { getPostPage } from "../../services/postsService";
import { EMPTY_PAGE, Page, Post } from "../../types";
import { formatDate } from "../../utils/time-util";
import useMessage from "../../hooks/useMessage";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { useNavigate } from "react-router-dom";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";
import { ConfirmDialog } from "../common/ConfirmDialog";

const columns: GridColDef[] = [
  {
    field: "title",
    headerName: "标题",
    headerAlign: "center",
    minWidth: 200,
  },
  {
    field: "content",
    headerName: "内容",
    headerAlign: "center",
    minWidth: 400,
  },
  // {
  //   field: "metadata",
  //   headerName: "元数据",
  //   minWidth: 200,
  // },
  {
    field: "version",
    headerName: "版本",
    headerAlign: "center",
    minWidth: 100,
  },
  {
    field: "createTime",
    headerName: "创建时间",
    headerAlign: "center",
    minWidth: 200,
    valueFormatter: (params: Date) => {
      return formatDate(params);
    },
  },
  {
    field: "...",
    headerName: "...",
    headerAlign: "center",
    minWidth: 100,
    renderCell: (params) => {
      const [open, setOpen] = useState(false);
      return (
        <Box
          sx={{
            display: "flex",
            alignItems: "center",
            height: "100%",
          }}
        >
          <NavIconButton
            aria-label="edit"
            color="primary"
            path={`/post/${params.id}`}
          >
            <EditIcon />
          </NavIconButton>
          <IconButton
            aria-label="delete"
            color="error"
            onClick={() => {
              setOpen(true);
            }}
          >
            <DeleteIcon />
          </IconButton>
          <ConfirmDialog
            open={open}
            onClose={() => {
              setOpen(false);
            }}
            onConfirm={() => {
              console.log("confirm");
              setOpen(false);
            }}
            title={"删除文章"}
            content={"确定要删除这篇文章吗?"}
          />
        </Box>
      );
    },
  },
];

interface NavIconButtonProps extends Omit<IconButtonProps, "onClick"> {
  children: React.ReactNode;
  path: string;
}

const NavIconButton: React.FC<NavIconButtonProps> = ({
  children,
  path,
  ...restProps
}) => {
  const navigate = useNavigate();
  return (
    <IconButton onClick={() => navigate(path)} {...restProps}>
      {children}
    </IconButton>
  );
};

export const PostPage = () => {
  const [posts, setPosts] = useState<Page<Post>>(EMPTY_PAGE);
  const message = useMessage();
  const navigate = useNavigate();

  useEffect(() => {
    getPostPage(DEFAULT_PAGE, DEFAULT_PAGE_SIZE)
      .then((data) => {
        setPosts(data);
      })
      .catch((error) => {
        message.error(formatErrorMessage(error));
      });
  }, [message.error]);

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
      <Box display={"flex"} justifyContent={"space-between"} mb={2}>
        <Typography variant="h5">文章列表</Typography>
        <Button
          type="button"
          variant="contained"
          onClick={() => navigate("/post/new")}
        >
          创建新文章
        </Button>
      </Box>
      <DataGrid
        columns={columns}
        rows={posts.data}
        rowCount={posts.total}
        initialState={{
          pagination: {
            paginationModel: {
              pageSize: DEFAULT_PAGE_SIZE,
            },
          },
        }}
        pageSizeOptions={[1, 5, 7]}
        paginationMode="server"
        autoHeight
        disableRowSelectionOnClick
        onPaginationModelChange={(newModel) => {
          getPostPage(newModel.page + 1, newModel.pageSize).then((data) => {
            setPosts(data);
          });
        }}
      />
    </Box>
  );
};
