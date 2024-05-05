import { transformResponse } from "./utils/transform-response";
import axios from "axios";
import { CommonResult, Page, Post } from "../types";
import { BASE_URL } from "../constants";
import {
  CreatePostReq,
  QueryPostPageReq,
  UpdatePostReq,
} from "./requests/posts";

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
