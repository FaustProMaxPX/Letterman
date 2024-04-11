import axios from "axios";
import { Page, Post } from "../types";
import { BASE_URL } from "../constants";

export const getPostPage = async (
  page: number,
  pageSize: number
): Promise<Page<Post>> => {
  const { data } = await axios.get<Page<Post>>(`${BASE_URL}/api/posts/list`, {
    params: { page, pageSize },
  });
  return data;
};
