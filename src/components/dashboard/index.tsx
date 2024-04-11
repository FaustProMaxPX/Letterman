import { Grid, Paper } from "@mui/material";
import React from "react";
import { Title } from "./Title";

export const Dashboard = () => {
  return (
    <React.Fragment>
      <Grid container spacing={3}>
        <Grid item xs={12} md={8} lg={9}>
          <Paper
            sx={{ p: 2, display: "flex", flexDirection: "column", height: 240 }}
          >
            <Title>提交记录</Title>
          </Paper>
        </Grid>
        <Grid item xs={12} md={4} lg={3}>
          <Paper
            sx={{
              p: 2,
              display: "flex",
              flexDirection: "column",
              height: 240,
            }}
          >
            <Title>总发布文章数</Title>
          </Paper>
        </Grid>
      </Grid>
    </React.Fragment>
  );
};
