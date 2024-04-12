import { Box, Button, Typography } from "@mui/material";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutline";
export const ErrorDisplay = () => (
  <Box
    sx={{
      display: "flex",
      justifyContent: "center",
      flexDirection: "column",
      alignItems: "center",
      mt: 20,
    }}
  >
    <ErrorOutlineIcon sx={{ fontSize: 120 }} color="error" />
    <Typography variant="h5" gutterBottom>
      哎呀，出错了！
    </Typography>
    <Box>
      <Button sx={{mr: 2}} variant="outlined" color="primary" href="/">
        返回首页
      </Button>
      <Button
        variant="outlined"
        color="secondary"
        onClick={() => window.location.reload()}
      >
        刷新页面
      </Button>
    </Box>
  </Box>
);
