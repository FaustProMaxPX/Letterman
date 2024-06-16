import { AccordionDetails } from "@mui/material";
import { BaseSyncRecord, GithubSyncRecord, Platform } from "../../types";

export const GithubAccordion = ({ repository, path }: GithubSyncRecord) => {
  return (
    <AccordionDetails>
      仓库：{repository} <br /> 路径：{path}
    </AccordionDetails>
  );
};

export const SyncAccordionDetail = (record: BaseSyncRecord) => {
  switch (record.platform) {
    case Platform.Github:
      return <GithubAccordion {...(record as GithubSyncRecord)} />;
  }
};
