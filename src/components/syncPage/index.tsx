import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import {
  Accordion,
  AccordionDetails,
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
import { PLATFORM_SET } from "../../constants";
import useMessage from "../../hooks/useMessage";
import { getLatestSyncRecords } from "../../services/postsService";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { BaseSyncRecord, Platform } from "../../types";
import { LoadingDisplay } from "../common/LoadingDisplay";
import { SyncAccordionDetail } from "./SyncAccordionDetails";
import { SyncButtonGroup } from "./SyncButtonGroup";

export const SyncPage = () => {
  const params = useParams();
  const id = params.id;
  const [records, setRecords] = useState<BaseSyncRecord[]>([]);
  const message = useMessage();
  const [loading, setLoading] = useState(false);
  useEffect(() => {
    setLoading(true);
    if (id !== undefined) {
      getLatestSyncRecords(id)
        .then((res) => {
          setRecords(res);
        })
        .catch((err) => {
          message.error(formatErrorMessage(err));
        })
        .finally(() => setLoading(false));
    }
  }, [id]);

  if (loading) {
    return <LoadingDisplay />;
  }

  // if (records.length === 0) {
  //   return <NotFoundDisplay text="当前文章暂时没有同步记录" />;
  // }
  const recordsMap = new Map(records.map((item) => [item.platform, item]));

  return (
    <>
      <Typography variant="h5">最近同步记录</Typography>
      <Grid container spacing={2}>
        {PLATFORM_SET.map((platform) => (
          <Grid item xs={12} sm={6} md={4} key={platform}>
            <SyncRecordCard
              id={id || ""}
              platform={platform}
              record={recordsMap.get(platform)}
            />
          </Grid>
        ))}
      </Grid>
    </>
  );
};

interface SyncRecordCardProps {
  id: string;
  platform: Platform;
  record?: BaseSyncRecord;
}

const SyncRecordCard = (props: SyncRecordCardProps) => {
  const { platform, record, id } = props;
  if (record === undefined) {
    return (
      <Card sx={{ mt: 2 }}>
        <CardContent>
          <Typography gutterBottom variant="h5" component="div">
            {platform}
          </Typography>
          暂无同步记录
        </CardContent>
        <Accordion>
          <AccordionSummary expandIcon={<ExpandMoreIcon />}>
            查看详情
          </AccordionSummary>
          <AccordionDetails>
            暂无同步记录
            <SyncButtonGroup id={id} platform={platform} first={true} />
          </AccordionDetails>
        </Accordion>
      </Card>
    );
  }
  return (
    <>
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
          <AccordionDetails>
            <SyncAccordionDetail {...record} />
            <SyncButtonGroup id={id} platform={platform} first={false} />
          </AccordionDetails>
        </Accordion>
      </Card>
    </>
  );
};
