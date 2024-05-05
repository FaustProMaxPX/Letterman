import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
} from "@mui/material";

interface ConfirmDialogProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title?: string;
  content: string;
}

export const ConfirmDialog = (props: ConfirmDialogProps) => (
  <Dialog open={props.open} onClose={props.onClose}>
    {props.title && (
      <DialogTitle id="alert-dialog-title">{props.title}</DialogTitle>
    )}
    <DialogContent id="alert-dialog-description">{props.content}</DialogContent>
    <DialogActions>
      <Button onClick={props.onConfirm}>确认</Button>
      <Button onClick={props.onClose}>取消</Button>
    </DialogActions>
  </Dialog>
);
