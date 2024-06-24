import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import {
  Accordion,
  AccordionSummary,
  Box,
  Card,
  CardContent,
  Chip,
  Grid,
  Link,
  Typography,
} from "@mui/material";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import useMessage from "../../hooks/useMessage";
import { getLatestSyncRecords } from "../../services/postsService";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { BaseSyncRecord } from "../../types";
import { SyncAccordionDetail } from "./SyncAccordionDetails";
import { NotFoundDisplay } from "../common/NotFoundDisplay";

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
  
  if (records.length === 0) {
    return <NotFoundDisplay text="当前文章暂时没有同步记录"/>
  }

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
      <Typography variant="h5">最近同步记录</Typography>
      <Card sx={{ mt: 2 }}>
        <CardContent>
          <Box sx={{ display: "flex", justifyContent: "space-between" }}>
            <Typography gutterBottom variant="h5" component="div">
              {record.platform}
            </Typography>
            <Chip
              label="查看所有同步记录"
              component="a"
              href={`/posts/sync/${record.post.id}/list`}
              clickable
            />
          </Box>
          <Typography>
            同步时间：{record.createTime.toLocaleString()}
          </Typography>
          <Typography>标题：{record.post.title}</Typography>
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
