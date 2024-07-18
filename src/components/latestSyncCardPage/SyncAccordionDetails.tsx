import { Typography } from "@mui/material";
import { BaseSyncRecord, GithubSyncRecord, Platform } from "../../types";

export const GithubAccordion = ({
  repository,
  path,
  ...record
}: GithubSyncRecord) => {
  return (
    <>
      仓库：{repository} <br /> 路径：{path} <br />{" "}
      {record.version === record.latestVersion && <Typography color={"#FF8000"}>当前已经是最新版本</Typography> }
    </>
  );
};

export const SyncAccordionDetail = (record: BaseSyncRecord) => {
  switch (record.platform) {
    case Platform.Github:
      return <GithubAccordion {...(record as GithubSyncRecord)} />;
  }
};
