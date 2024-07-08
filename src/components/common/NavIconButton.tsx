import { IconButton, IconButtonProps } from "@mui/material";
import { useNavigate } from "react-router-dom";

interface NavIconButtonProps extends Omit<IconButtonProps, "onClick"> {
  children: React.ReactNode;
  path: string;
}

export const NavIconButton: React.FC<NavIconButtonProps> = ({
  children,
  path,
  ...restProps
}) => {
  const navigate = useNavigate();
  return (
    <IconButton onClick={() => navigate(path)} {...restProps}>
      {children}
    </IconButton>
  );
};
