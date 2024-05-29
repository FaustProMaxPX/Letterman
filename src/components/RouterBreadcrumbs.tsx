import { Breadcrumbs, Link, LinkProps, Typography } from "@mui/material";
import { useLocation } from "react-router-dom";
import { BREADCRUMB_NAME_MAP } from "../constants";
import { Link as RouterLink } from "react-router-dom";

interface LinkRouterProps extends LinkProps {
  to: string;
  replace?: boolean;
}

const LinkRouter = (props: LinkRouterProps) => {
  return <Link {...props} component={RouterLink} />;
};

export const RouterBreadcurmbs = () => {
  const location = useLocation();
  const pathnames = location.pathname.split("/").filter((x) => x);

  return (
    <Breadcrumbs aria-label="breadcrumb" sx={{ mb: 1 }}>
      <LinkRouter underline="hover" color="inherit" to="/">
        Home
      </LinkRouter>
      {pathnames.map((_value, index) => {
        const last = index === pathnames.length - 1;
        const to = `/${pathnames.slice(0, index + 1).join("/")}`;

        return last ? (
          <Typography color="text.primary" key={to}>
            {BREADCRUMB_NAME_MAP[to]}
          </Typography>
        ) : (
          <LinkRouter underline="hover" color="inherit" to={to} key={to}>
            {BREADCRUMB_NAME_MAP[to]}
          </LinkRouter>
        );
      })}
    </Breadcrumbs>
  );
};
