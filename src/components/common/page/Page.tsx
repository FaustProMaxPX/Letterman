/* eslint-disable @typescript-eslint/no-explicit-any */
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { createContext, useEffect, useState } from "react";
import { DEFAULT_PAGE, DEFAULT_PAGE_SIZE } from "../../../constants";
import useMessage from "../../../hooks/useMessage";
import { formatErrorMessage } from "../../../services/utils/transform-response";
import { EMPTY_PAGE, Page } from "../../../types";

interface PageContextProps {
  setData: React.Dispatch<React.SetStateAction<Page<any>>>;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
  attributes?: Map<string, any>;
}

export const PageContext = createContext<PageContextProps | null>(null);

export interface BasePageProps {
  colDef: GridColDef[];
  onPageChange: (page: number, pageSize: number) => Promise<Page<any>>;
  initialPageSize?: number;
  attributes?: Map<string, any>;
}

export const BasePage = (props: BasePageProps) => {
  const [data, setData] = useState<Page<any>>(EMPTY_PAGE);
  const [loading, setLoading] = useState(true);
  const context: PageContextProps = {
    setData,
    setLoading,
    attributes: props.attributes,
  };

  const message = useMessage();

  useEffect(() => {
    setLoading(true);
    props
      .onPageChange(DEFAULT_PAGE, props.initialPageSize || DEFAULT_PAGE_SIZE)
      .then((data) => {
        setData(data);
      })
      .catch((err) => {
        message.error(formatErrorMessage(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }, [props]);

  return (
    <>
      <PageContext.Provider value={context}>
        <DataGrid
          columns={props.colDef}
          rows={data.data}
          rowCount={data.total}
          loading={loading}
          initialState={{
            pagination: {
              paginationModel: {
                pageSize: props.initialPageSize || DEFAULT_PAGE_SIZE,
              },
            },
          }}
          pageSizeOptions={[1, 5, 10]}
          paginationMode="server"
          autoHeight
          disableRowSelectionOnClick
          onPaginationModelChange={(newModel) => {
            setLoading(true);
            props
              .onPageChange(newModel.page + 1, newModel.pageSize)
              .then((data) => {
                setData(data);
                setLoading(false);
              })
              .catch((error) => {
                message.error(formatErrorMessage(error));
              })
              .finally(() => {
                setLoading(false);
              });
          }}
          sx={{ justifyContent: "center" }}
        ></DataGrid>
      </PageContext.Provider>
    </>
  );
};
