export interface Post {
  id: string;
  postId: string;
  title: string;
  content: string;
  metadata: JSON;
  version: number;
  preVersion: number;
  createTime: Date;
}

export type BasePost = Omit<Post, "content">;

export interface Page<T> {
  total: number;
  prev: number;
  next: number;
  data: Array<T>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const EMPTY_PAGE: Page<any> = {
  total: 0,
  prev: 0,
  next: 0,
  data: [],
};

export interface CommonResult<T> {
  success: boolean;
  code: number;
  message: string;
  data: T;
}

export interface BaseSyncRecord {
  post: Post;
  platform: Platform;
  url: string;
  createTime: Date;
}

export interface GithubSyncRecord extends BaseSyncRecord {
  repository: string;
  path: string;
}

export enum Platform {
  Github = "Github",
}
