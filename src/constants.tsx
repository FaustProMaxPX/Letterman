import gfm from "@bytemd/plugin-gfm";
import { Platform } from "./types";

export const BASE_URL = "http://localhost:8080";

export const TIME_DISPLAY_FORMAT = "yyyy-MM-dd HH:mm:ss";

export const DEFAULT_PAGE = 1;

export const DEFAULT_PAGE_SIZE = 10;

export const BREADCRUMB_NAME_MAP: { [key: string]: string } = {
  "/": "Home",
  "/posts": "Posts",
  "/posts/new": "New",
  "/posts/:id": "Edit",
  "/posts/sync/:id": "Latest Sync",
  "/posts/sync/:id/list": "Sync",
  "/posts/sync/:postId/detail/:id": "Sync Detail",
  "/posts/history/:postId": "History",
};

export const PLATFORM_SET = [Platform.Github];

export const BYTEMD_PLUGINS = [gfm()];
