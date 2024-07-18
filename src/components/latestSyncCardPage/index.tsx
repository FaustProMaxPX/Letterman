import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Box,
  Button,
  Card,
  CardContent,
  Grid,
  Link,
  Typography,
} from "@mui/material";
import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { PLATFORM_SET } from "../../constants";
import useMessage from "../../hooks/useMessage";
import { getLatestSyncRecords } from "../../services/postsService";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { BaseSyncRecord, Platform } from "../../types";
import { LoadingDisplay } from "../common/LoadingDisplay";
import { SyncAccordionDetail } from "./SyncAccordionDetails";
import { SyncButtonGroup } from "./SyncButtonGroup";

export const LatestSyncCardPage = () => {
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
  const navigate = useNavigate();

  if (loading) {
    return <LoadingDisplay />;
  }

  const recordsMap = new Map(records.map((item) => [item.platform, item]));

  return (
    <>
      <Box display={"flex"} justifyContent={"space-between"} sx={{ mt: 1 }}>
        <Typography variant="h5">最近同步记录</Typography>
        <Button
          type="button"
          variant="contained"
          onClick={() => navigate(`/posts/sync/${id}/list`)}
        >
          查看历史同步记录
        </Button>
      </Box>
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
        <Box sx={{ display: "flex" }}>
          <Box sx={{ flex: "1 1 auto" }}>
            <CardContent>
              <Box sx={{ display: "flex", justifyContent: "space-between" }}>
                <Typography gutterBottom variant="h5" component="div">
                  {record.platform}
                </Typography>
              </Box>
              <Typography>
                同步时间：{record.createTime.toLocaleString()}
              </Typography>
              <Typography>标题：{record.post.title}</Typography>
              <Link href={record.url}>前往文章</Link> <br />
            </CardContent>
          </Box>
          <Box sx={{ flex: "0 0 auto" }}>
            {record.version === record.latestVersion && (
              <img width="50px" height="50px" src="/src/assets/latest.svg" />
            )}
          </Box>
        </Box>
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
