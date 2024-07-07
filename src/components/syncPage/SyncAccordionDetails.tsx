import { BaseSyncRecord, GithubSyncRecord, Platform } from "../../types";

export const GithubAccordion = ({
  repository,
  path,
  ...record
}: GithubSyncRecord) => {
  return (
    <>
      仓库：{repository} <br /> 路径：{path} <br /> 同步版本：{record.version}{" "}
      <br /> 最新版本：{record.latestVersion}
    </>
  );
};

export const SyncAccordionDetail = (record: BaseSyncRecord) => {
  switch (record.platform) {
    case Platform.Github:
      return <GithubAccordion {...(record as GithubSyncRecord)} />;
  }
};
