import {
  Button,
  ButtonGroup,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
} from "@mui/material";
import { useState } from "react";
import { FormContext, useForm } from "../../hooks/useForm";
import useMessage from "../../hooks/useMessage";
import { forcePull, forcePush, synchronize } from "../../services/postsService";
import { BaseSyncReq, GithubSyncReq } from "../../services/requests/posts";
import { formatErrorMessage } from "../../services/utils/transform-response";
import { Platform } from "../../types";
import { FormField } from "../common/form/FormField";

export interface SyncButtonGroupProps {
  id: string;
  platform: Platform;
  first: boolean;
}

export const SyncButtonGroup = (props: SyncButtonGroupProps) => {
  const [open, setOpen] = useState(false);
  const [type, setType] = useState<"sync" | "push" | "pull">("sync");
  const message = useMessage();
  return (
    <>
      <ButtonGroup
        variant="outlined"
        sx={{ display: "flex", justifyContent: "center", mt: 2 }}
      >
        <Button
          onClick={() => {
            if (props.first) {
              setType("sync");
              setOpen(true);
              return;
            }
            synchronize(props.id, { platform: props.platform })
              .then(() => {
                setTimeout(() => {
                  window.location.reload();
                }, 1000);
              })
              .catch((err) => {
                message.error(formatErrorMessage(err));
              });
          }}
        >
          Sync
        </Button>
        <Button
          onClick={() => {
            if (props.first) {
              setType("pull");
              setOpen(true);
              return;
            }
            forcePull(props.id, { platform: props.platform })
              .then(() => {
                message.success(`同步成功`);
                setTimeout(() => {
                  window.location.reload();
                }, 1000);
              })
              .catch((err) => {
                message.error(formatErrorMessage(err));
              });
          }}
        >
          Pull
        </Button>
        <Button
          onClick={() => {
            if (props.first) {
              setType("push");
              setOpen(true);
              return;
            }
            forcePush(props.id, { platform: props.platform })
              .then(() => {
                message.success(`同步成功`);
                setTimeout(() => {
                  window.location.reload();
                }, 1000);
              })
              .catch((err) => {
                message.error(formatErrorMessage(err));
              });
          }}
        >
          Push
        </Button>
      </ButtonGroup>
      <SyncDialog
        id={props.id}
        open={open}
        onClose={() => setOpen(false)}
        platform={props.platform}
        type={type}
      />
    </>
  );
};

interface SyncDialogProps {
  id: string;
  platform: Platform;
  type: "push" | "pull" | "sync";
  open: boolean;
  onClose: () => void;
}

const SyncDialog = (props: SyncDialogProps) => {
  let onSubmit;
  const message = useMessage();
  switch (props.type) {
    case "sync":
      onSubmit = (id: string, req: BaseSyncReq) => {
        synchronize(id, req)
          .then(() => {
            message.success("同步成功");
            setTimeout(() => window.location.reload(), 1000);
          })
          .catch((err) => {
            message.error(formatErrorMessage(err));
          });
      };
      break;
    case "pull":
      onSubmit = (id: string, req: BaseSyncReq) => {
        forcePull(id, req)
          .then(() => {
            message.success("同步成功");
            setTimeout(() => window.location.reload(), 1000);
          })
          .catch((err) => {
            message.error(formatErrorMessage(err));
          });
      };
      break;
    case "push":
      onSubmit = (id: string, req: BaseSyncReq) => {
        forcePush(id, req)
          .then(() => {
            message.success("同步成功");
            setTimeout(() => window.location.reload(), 1000);
          })
          .catch((err) => {
            message.error(formatErrorMessage(err));
          });
      };
      break;
  }
  switch (props.platform) {
    case Platform.Github:
      return (
        <GithubSyncDialog
          id={props.id}
          open={props.open}
          onClose={props.onClose}
          onSubmit={onSubmit}
          type={props.type}
        />
      );
  }
};

interface GithubSyncDialogProps {
  id: string;
  open: boolean;
  onClose: () => void;
  onSubmit: (id: string, req: GithubSyncReq) => void;
  type: "sync" | "push" | "pull";
}

export const GithubSyncDialog = (props: GithubSyncDialogProps) => {
  const form = useForm({
    initialValue: {
      repository: "",
      path: "",
    },
    onSubmit: (values) => {
      const req: GithubSyncReq = {
        repository: values.repository,
        path: values.path,
        platform: Platform.Github,
      };

      props.onSubmit(props.id, req);
      form.reset();
      props.onClose();
    },
    validate: () => {
      return {};
    },
  });
  return (
    <Dialog
      open={props.open}
      onClose={props.onClose}
      PaperProps={{
        component: "form",
        onSubmit: form.handleSubmit,
      }}
    >
      <DialogTitle>{props.type}</DialogTitle>
      <FormContext.Provider value={form}>
        <DialogContent>
          <FormField id="repository" label="Repository" />
          <FormField id="path" label="Path" />
        </DialogContent>
      </FormContext.Provider>
      <DialogActions>
        <Button type="submit">提交</Button>
        <Button onClick={props.onClose}>取消</Button>
      </DialogActions>
    </Dialog>
  );
};
