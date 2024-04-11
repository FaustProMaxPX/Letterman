export interface Post {
  id: string;
  title: string;
  content: string;
  metadata: unknown;
  version: number;
  preVersion: number;
  createTime: Date;
}

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
  code: number;
  msg: string;
  data: T;
}
