import { useState, useEffect } from "react";
import { getPostPage } from "../services/postsService";
import { Post } from "../types";
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { Box } from "@mui/material";

const columns: GridColDef[] = [
  {
    field: "title",
    headerName: "标题",
    minWidth: 200,
    flex: 1,
  },
  {
    field: "content",
    headerName: "内容",
    minWidth: 400,
    flex: 1,
  },
  {
    field: "metadata",
    headerName: "元数据",
    minWidth: 200,
  },
  {
    field: "version",
    headerName: "版本",
    minWidth: 100,
  },
];

export const PostPage = () => {
  const [posts, setPosts] = useState<Array<Post>>([]);
  useEffect(() => {
    getPostPage(1, 10).then((data) => setPosts(data.data));
  }, []);
  return (
    <Box
      sx={{ minHeight: 300, width: "100%", flexGrow: 1, overflow: "hidden" }}
    >
      <DataGrid
        columns={columns}
        rows={posts}
        initialState={{
          pagination: { paginationModel: { page: 1, pageSize: 10 } },
        }}
        pageSizeOptions={[5, 10]}
        checkboxSelection
        autoHeight
        disableRowSelectionOnClick
      ></DataGrid>
    </Box>
  );
};
