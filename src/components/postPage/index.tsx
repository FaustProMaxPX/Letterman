import { Box } from "@mui/material";
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { useEffect, useState } from "react";
import { DEFAULT_PAGE, DEFAULT_PAGE_SIZE } from "../../constants";
import { getPostPage } from "../../services/postsService";
import { EMPTY_PAGE, Page, Post } from "../../types";
import { formatDate } from "../../utils/time-util";
import useMessage from "../../hooks/useMessage";

const columns: GridColDef[] = [
  {
    field: "title",
    headerName: "标题",
    minWidth: 200,
  },
  {
    field: "content",
    headerName: "内容",
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
    minWidth: 100,
  },
  {
    field: "createTime",
    headerName: "创建时间",
    minWidth: 200,
    valueFormatter: (params: string) => {
      return formatDate(new Date(params));
    },
  },
];

export const PostPage = () => {
  const [posts, setPosts] = useState<Page<Post>>(EMPTY_PAGE);
  const openMessage = useMessage();
  useEffect(() => {
    getPostPage(DEFAULT_PAGE, DEFAULT_PAGE_SIZE)
      .then((data) => {
        setPosts(data);
      })
      .catch((error) => {
        openMessage(error.message, 3000);
      });
  }, [openMessage]);

  return (
    <Box
      sx={{ minHeight: 300, width: "100%", flexGrow: 1, overflow: "hidden" }}
    >
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
        pageSizeOptions={[1, 5, 10]}
        paginationMode="server"
        checkboxSelection
        autoHeight
        disableRowSelectionOnClick
        onPaginationModelChange={(newModel) => {
          getPostPage(newModel.page, newModel.pageSize).then((data) => {
            setPosts(data);
          });
        }}
      ></DataGrid>
    </Box>
  );
};
