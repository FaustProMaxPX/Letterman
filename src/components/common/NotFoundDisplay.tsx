import { Box, Typography } from "@mui/material";
export const NotFoundDisplay = ({ text }: { text: string }) => (
  <Box
    sx={{
      display: "flex",
      justifyContent: "center",
      flexDirection: "column",
      alignItems: "center",
      mt: 20,
    }}
  >
    <img src="/src/assets/404.svg" />
    <Typography variant="h5" gutterBottom>
      {text}
    </Typography>
  </Box>
);
