import { Post } from "../../types";

export type CreatePostReq = Omit<
  Post,
  "id" | "createTime" | "version" | "preVersion"
>;

export type UpdatePostReq = Omit<Post, "createTime" | "version" | "preVersion">