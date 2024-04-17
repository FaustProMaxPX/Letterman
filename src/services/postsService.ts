import { transformResponse } from "./utils/transform-response";
import axios from "axios";
import { CommonResult, Page, Post } from "../types";
import { BASE_URL } from "../constants";
import { CreatePostReq } from "./requests/posts";

export const getPostPage = async (
  page: number,
  pageSize: number
): Promise<Page<Post>> => {
  const data = await axios.get<CommonResult<Page<Post>>>(
    `${BASE_URL}/api/posts/list`,
    {
      params: { page, pageSize },
    }
  );
  return transformResponse(data);
};

export const createPost = async (post: CreatePostReq) => {
  const data = await axios.post<CommonResult<null>>(`${BASE_URL}/api/posts`, {
    ...post,
    metadata: JSON.stringify(post.metadata),
  });
  return transformResponse(data);
};

export const getPost = async (id: string) => {
  const data = await axios.get<CommonResult<Post>>(
    `${BASE_URL}/api/post/${id}`
  );
  return transformResponse(data);
};
