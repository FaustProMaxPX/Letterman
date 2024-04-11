export interface Post {
  id: string;
  title: string;
  content: string;
  metadata: unknown;
  version: number;
  preVersion: number;
  createTime: Date
}

export interface Page<T> {
  total: number;
  prev: number;
  next: number;
  data: Array<T>;
}
