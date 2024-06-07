import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import {
  Box,
  Button,
  IconButton,
  IconButtonProps,
  Typography,
} from "@mui/material";
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { createContext, useContext, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { DEFAULT_PAGE, DEFAULT_PAGE_SIZE } from "../../constants";
import useMessage from "../../hooks/useMessage";
import { deletePost, getPostPage } from "../../services/postsService";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { EMPTY_PAGE, Page, Post } from "../../types";
import { formatDate } from "../../utils/time-util";
import { ConfirmDialog } from "../common/ConfirmDialog";

interface GridContextProps {
  setData: React.Dispatch<React.SetStateAction<Page<Post>>>;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
  all: boolean;
}

const GridContext = createContext<GridContextProps | null>(null);

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
    minWidth: 100,
    align: "center",
    renderCell: (params) => <OptionCell id={params.row.id} />,
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

const OptionCell = ({ id }: { id: string }) => {
  const [open, setOpen] = useState(false);
  const message = useMessage();
  const ctx = useContext(GridContext);

  return (
    <Box
      sx={{
        display: "flex",
        alignItems: "center",
        height: "100%",
      }}
    >
      <NavIconButton aria-label="edit" color="primary" path={`/posts/${id}`}>
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
          deletePost(id)
            .then(() => {
              message.info("删除成功");
              getPostPage({
                page: DEFAULT_PAGE,
                pageSize: DEFAULT_PAGE_SIZE,
                all: ctx?.all,
              })
                .then((data) => ctx?.setData(data))
                .catch((error) => {
                  message.error(formatErrorMessage(error));
                });
            })
            .catch((error) => {
              message.error(formatErrorMessage(error));
            })
            .finally(() => {
              setOpen(false);
            });
        }}
        title={"删除文章"}
        content={"确定要删除这篇文章吗?"}
      />
    </Box>
  );
};

export const PostPage = () => {
  const [posts, setPosts] = useState<Page<Post>>(EMPTY_PAGE);
  const [all, setAll] = useState(false);
  const message = useMessage();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);

  const formCtx: GridContextProps = {
    setData: setPosts,
    setLoading: setLoading,
    all: all,
  };

  useEffect(() => {
    setLoading(true);
    getPostPage({ page: DEFAULT_PAGE, pageSize: DEFAULT_PAGE_SIZE, all })
      .then((data) => {
        setPosts(data);
        setLoading(false);
      })
      .catch((error) => {
        message.error(formatErrorMessage(error));
        setLoading(false);
      });
  }, [message.error, all]);

  return (
    <GridContext.Provider value={formCtx}>
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
          <Box display={"flex"} justifyContent={"flex-end"} gap={1}>
            <Button
              type="button"
              variant="contained"
              onClick={() => navigate("/posts/new")}
            >
              创建新文章
            </Button>
            {!all && (
              <Button
                type="button"
                variant="contained"
                onClick={() => {
                  setAll(true);
                }}
              >
                查询所有文章
              </Button>
            )}
            {all && (
              <Button
                type="button"
                variant="contained"
                onClick={() => setAll(false)}
              >
                仅显示最新版文章
              </Button>
            )}
          </Box>
        </Box>
        <DataGrid
          columns={columns}
          rows={posts.data}
          rowCount={posts.total}
          loading={loading}
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
            setLoading(true);
            getPostPage({
              page: newModel.page + 1,
              pageSize: newModel.pageSize,
              all: all,
            })
              .then((data) => {
                setPosts(data);
                setLoading(false);
              })
              .catch((error) => {
                message.error(formatErrorMessage(error));
              });
          }}
          sx={{ justifyContent: "center" }}
        />
      </Box>
    </GridContext.Provider>
  );
};
