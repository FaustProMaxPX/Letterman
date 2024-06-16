import { Breadcrumbs, Link, LinkProps, Typography } from "@mui/material";
import { Link as RouterLink, useLocation } from "react-router-dom";
import { BREADCRUMB_NAME_MAP } from "../constants";

interface LinkRouterProps extends LinkProps {
  to: string;
  replace?: boolean;
}

const LinkRouter = (props: LinkRouterProps) => {
  return <Link {...props} component={RouterLink} />;
};

const getBreadcrumbName = (to: string) => {
  const pathNames = Object.keys(BREADCRUMB_NAME_MAP);
  console.log("to", to);
  
  for (const path of pathNames) {
    if (path == to) {
      return BREADCRUMB_NAME_MAP[path];
    }
    const match = path.match(/\/:([^/]+)/);
    const segment = path.split("/:")[0];
    console.log(match);
    
    if (match && to.startsWith(segment)) {
      console.log(BREADCRUMB_NAME_MAP[path]);

      return BREADCRUMB_NAME_MAP[path];
    }
  }
  return to;
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
            {getBreadcrumbName(to)}
          </Typography>
        ) : (
          <LinkRouter underline="hover" color="inherit" to={to} key={to}>
            {getBreadcrumbName(to)}
          </LinkRouter>
        );
      })}
    </Breadcrumbs>
  );
};
