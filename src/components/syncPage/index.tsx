import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import {
  Accordion,
  AccordionSummary,
  Card,
  CardContent,
  Grid,
  Link,
  Typography,
} from "@mui/material";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import useMessage from "../../hooks/useMessage";
import { getLatestSyncRecords } from "../../services/postsService";
import { BaseSyncRecord } from "../../types";
import { SyncAccordionDetail } from "./SyncAccordionDetails";
import { formatErrorMessage } from "../../services/utils/transform-response";

// const records: BaseSyncRecord[] = [
//   {
//     platform: Platform.Github,
//     createTime: new Date(),
//     url: "https://github.com/xxx",
//     repository: "xxx",
//     path: "xxx",
//     post: {
//       title: "xxx",
//       content: "xxx",
//       metadata: JSON.parse('{"key":"value"}'),
//       version: 0,
//       preVersion: 0,
//       id: "",
//       createTime: new Date(),
//     },
//   } as GithubSyncRecord,
// ];

export const SyncPage = () => {
  const params = useParams();
  const id = params.id;
  const [records, setRecords] = useState<BaseSyncRecord[]>([]);
  const message = useMessage();
  useEffect(() => {
    if (id !== undefined) {
      getLatestSyncRecords(id)
        .then((res) => {
          console.log(res);
          
          setRecords(res);
        })
        .catch((err) => {
          message.error(formatErrorMessage(err));
        });
    }
  }, [id]);

  return (
    <Grid container spacing={2}>
      {records.map((record) => (
        <Grid item xs={12} sm={6} md={4} key={record.post.id}>
          <SyncRecordCard record={record} />
        </Grid>
      ))}
    </Grid>
  );
};

interface SyncRecordCardProps {
  record: BaseSyncRecord;
}

const SyncRecordCard = (props: SyncRecordCardProps) => {
  const { record } = props;
  return (
    <>
      <Card>
        <CardContent>
          <Typography gutterBottom variant="h5" component="div">
            {record.post.title}
          </Typography>
          <Typography>同步时间：{record.platform}</Typography>
          <Typography>平台：{record.platform}</Typography>
          <Link href={record.url}>前往文章</Link>
        </CardContent>
        <Accordion>
          <AccordionSummary expandIcon={<ExpandMoreIcon />}>
            查看详情
          </AccordionSummary>
          <SyncAccordionDetail {...record} />
        </Accordion>
      </Card>
    </>
  );
};
