export const BASE_URL = "http://localhost:8080";

export const TIME_DISPLAY_FORMAT = "yyyy-MM-dd HH:mm:ss";

export const DEFAULT_PAGE = 1;

export const DEFAULT_PAGE_SIZE = 7;

export const BREADCRUMB_NAME_MAP: { [key: string]: string } = {
  "/": "Home",
  "/posts": "Posts",
  "/posts/:id": "Edit",
  "/posts/new": "New",
};
