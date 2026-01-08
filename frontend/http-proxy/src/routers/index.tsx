import { createBrowserRouter } from "react-router-dom";
import Layout from "@/layout/Layout"; // 你的布局组件
import TableTest from "@/pages/TableTest.tsx";
import App from "../App.tsx";
import S3Setting from "@/pages/S3Setting.tsx";
import ProxySetting from "@/pages/ProxySetting.tsx";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      {
        index: true,
        path: "resources",
        element: <TableTest />,
      },
      {
        path: "resources/s3/new",
        element: <S3Setting />,
      },
      {
        path: "resources/proxy/new",
        element: <ProxySetting />,
      },
    ],
  },
]);
