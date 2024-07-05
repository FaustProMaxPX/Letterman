import { Box, CircularProgress } from "@mui/material";

export const LoadingDisplay = () => (
  <Box
    sx={{
      display: "flex",
      justifyContent: "center",
      alignItems: "center",
      mt: 20,
    }}
  >
    <CircularProgress />
  </Box>
);
