/* eslint-disable @typescript-eslint/no-explicit-any */
import { AxiosResponse } from "axios";
import { CommonResult } from "../../types";

export const transformResponse = <T = any>(
  response: AxiosResponse<CommonResult<T>>
): T => {
  if (response.status >= 200 && response.status < 300) {
    const res = response.data;
    const err = response as unknown as Error;
    if (res?.success) {
      return res?.data || ({} as any);
    }
    throw new Error(res.message || err.message || "Unknown Error");
  } else {
    throw new Error(`${response.status}: ${response.statusText}`);
  }
};
