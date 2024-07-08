import * as React from "react";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import DashboardIcon from "@mui/icons-material/Dashboard";
import ArticleIcon from "@mui/icons-material/Article";
import { NavLink } from "./components/common/NavLink";
import SyncIcon from "@mui/icons-material/Sync";

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
    <NavLink to={"/sync"}>
      <ListItemButton>
        <ListItemIcon>
          <SyncIcon />
        </ListItemIcon>
        <ListItemText primary="Sync" />
      </ListItemButton>
    </NavLink>
  </React.Fragment>
);
