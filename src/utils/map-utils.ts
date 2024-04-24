export const jsonToMap = (obj: JSON | undefined): Map<string, string> => {
  if (obj === undefined) {
    return new Map();
  }
  const ret = new Map();
  for (const key of Object.keys(obj)) {
    ret.set(key, obj[key]);
  }
  return ret;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const mapToJson = (obj: Map<any, any>) => {
  const str = JSON.stringify(Object.fromEntries(obj));
  return JSON.parse(str);
};
