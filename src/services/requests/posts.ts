import { Platform, Post } from "../../types";

export type CreatePostReq = Omit<
  Post,
  "id" | "createTime" | "version" | "preVersion" | "postId"
>;

export type UpdatePostReq = Omit<
  Post,
  "createTime" | "version" | "preVersion" | "postId"
>;

export interface PageReq {
  page: number;
  pageSize: number;
}

export interface QueryPostPageReq extends PageReq {
  all?: boolean;
}

export interface BaseSyncReq {
  platform: Platform;
}

export interface GithubSyncReq extends BaseSyncReq {
  repository?: string;
  path?: string;
}

export interface SyncPageReq {
  page: number;
  pageSize: number;
  platform: Platform;
}

export interface RevertReq {
  id: string;
}
