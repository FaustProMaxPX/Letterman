import axios from "axios";
import { BASE_URL } from "../constants";
import { BaseSyncRecord, CommonResult, Page, Post } from "../types";
import {
  BaseSyncReq,
  CreatePostReq,
  QueryPostPageReq,
  RevertReq,
  SyncPageReq,
  UpdatePostReq,
} from "./requests/posts";
import { transformResponse } from "./utils/transform-response";

export const getPostPage = async ({
  page,
  pageSize,
  all,
}: QueryPostPageReq): Promise<Page<Post>> => {
  const data = await axios.get<CommonResult<Page<Post>>>(
    `${BASE_URL}/api/post/list`,
    {
      params: { page, pageSize, all },
    }
  );
  data.data.data.data.forEach((post) => {
    post.createTime = new Date(post.createTime);
  });
  return transformResponse(data);
};

export const createPost = async (post: CreatePostReq) => {
  const data = await axios.post<CommonResult<null>>(`${BASE_URL}/api/post`, {
    ...post,
    metadata: JSON.stringify(post.metadata),
  });
  return transformResponse(data);
};

export const getPost = async (id: string) => {
  const data = await axios.get<CommonResult<Post>>(
    `${BASE_URL}/api/post/${id}`
  );
  data.data.data.createTime = new Date(data.data.data.createTime);
  return transformResponse(data);
};

export const updatePost = async (post: UpdatePostReq) => {
  const data = await axios.put<CommonResult<null>>(`${BASE_URL}/api/post`, {
    ...post,
    metadata: JSON.stringify(post.metadata),
  });
  return transformResponse(data);
};

export const deletePost = async (id: string) => {
  const data = await axios.delete<CommonResult<null>>(
    `${BASE_URL}/api/post/${id}`
  );
  return transformResponse(data);
};

export const getLatestSyncRecords = async (id: string) => {
  const data = await axios.get<CommonResult<BaseSyncRecord[]>>(
    `${BASE_URL}/api/post/sync/${id}/records/latest`
  );
  data.data.data.forEach((record) => {
    record.createTime = new Date(record.createTime);
  });
  return transformResponse(data);
};

export const synchronize = async (id: string, req: BaseSyncReq) => {
  const data = await axios.put<CommonResult<null>>(
    `${BASE_URL}/api/post/sync/${id}/synchronize`,
    req
  );
  return transformResponse(data);
};

export const forcePush = async (id: string, req: BaseSyncReq) => {
  const data = await axios.put<CommonResult<null>>(
    `${BASE_URL}/api/post/sync/${id}/push`,
    req
  );
  return transformResponse(data);
};

export const forcePull = async (id: string, req: BaseSyncReq) => {
  const data = await axios.put<CommonResult<null>>(
    `${BASE_URL}/api/post/sync/${id}/pull`,
    req
  );
  return transformResponse(data);
};

export const getSyncList = async (id: string, req: SyncPageReq) => {
  const data = await axios.get<CommonResult<Page<BaseSyncRecord>>>(
    `${BASE_URL}/api/post/sync/${id}/records`,
    {
      params: {
        page: req.page,
        pageSize: req.pageSize,
        platform: req.platform,
      },
    }
  );
  return transformResponse(data);
};

export const revertPost = async (req: RevertReq) => {
  const data = await axios.put<CommonResult<null>>(
    `${BASE_URL}/api/post/sync/revert`,
    req
  );
  return transformResponse(data);
};
