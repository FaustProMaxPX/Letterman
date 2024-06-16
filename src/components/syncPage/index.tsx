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
import { BaseSyncRecord, GithubSyncRecord, Platform } from "../../types";
import { SyncAccordionDetail } from "./SyncAccordionDetails";

const records: BaseSyncRecord[] = [
  {
    platform: Platform.Github,
    createTime: new Date(),
    url: "https://github.com/xxx",
    repository: "xxx",
    path: "xxx",
    post: {
      title: "xxx",
      content: "xxx",
      metadata: JSON.parse('{"key":"value"}'),
      version: 0,
      preVersion: 0,
      id: "",
      createTime: new Date(),
    },
  } as GithubSyncRecord,
];

export const SyncPage = () => {
  // const [record, setRecord] = useState<BaseSyncRecord[]>([]);
  // useEffect(() => {
  //   first;
  
  //   return () => {
  //     second;
  //   };
  // }, [third]);
  

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
