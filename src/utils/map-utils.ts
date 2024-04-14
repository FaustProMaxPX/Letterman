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
