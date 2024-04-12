import * as React from "react";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import DashboardIcon from "@mui/icons-material/Dashboard";
import ArticleIcon from "@mui/icons-material/Article";
import { Link } from "react-router-dom";
import { styled } from "@mui/material";

// eslint-disable-next-line react-refresh/only-export-components
const NavLink = styled(Link)({
  textDecoration: "none",
  color: "inherit",
});

export const mainListItems = (
  <React.Fragment>
    <NavLink to={"/"}>
      <ListItemButton>
        <ListItemIcon>
          <DashboardIcon />
        </ListItemIcon>
        <ListItemText primary="Dashboard" />
      </ListItemButton>
    </NavLink>
    <NavLink to={"/posts"}>
      <ListItemButton>
        <ListItemIcon>
          <ArticleIcon />
        </ListItemIcon>
        <ListItemText primary="Posts" />
      </ListItemButton>
    </NavLink>
  </React.Fragment>
);
