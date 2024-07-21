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

const getBreadcrumbName = (pathname: string) => {
  for (const route in BREADCRUMB_NAME_MAP) {
    const regex = new RegExp(`^${route.replace(/:[^\s/]+/g, "\\d+")}$`);
    if (regex.test(pathname)) {
      return BREADCRUMB_NAME_MAP[route];
    }
  }
  return null;
};

export const RouterBreadcurmbs = () => {
  const location = useLocation();
  const pathnames = location.pathname.split("/").filter((x) => x);
  const getPathname = (index: number) => {
    return `/${pathnames.slice(0, index + 1).join("/")}`;
  };
  return (
    <Breadcrumbs aria-label="breadcrumb" sx={{ mb: 1 }}>
      <LinkRouter underline="hover" color="inherit" to="/">
        Home
      </LinkRouter>

      {pathnames.map((_value, index) => {
        const to = getPathname(index);
        const breadcrumbName = getBreadcrumbName(to);
        const isLast = index === pathnames.length - 1;

        if (breadcrumbName) {
          return isLast ? (
            <Typography color="text.primary" key={to}>
              {breadcrumbName}
            </Typography>
          ) : (
            <LinkRouter underline="hover" color="inherit" key={to} to={to}>
              {breadcrumbName}
            </LinkRouter>
          );
        }
        return null;
      })}
    </Breadcrumbs>
  );
};
